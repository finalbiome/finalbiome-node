#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_std::prelude::*;

mod functions;
mod impl_fungible_assets;
mod types;

use pallet_support::{
  AccountIdOf, CheckedAdd, CheckedSub, DispatchResultAs, FungibleAssetBalance, SaturatingAdd,
  SaturatingSub,
};
pub use types::*;

use frame_support::{
  log,
  pallet_prelude::*,
  traits::{
    tokens::{DepositConsequence, WithdrawConsequence},
    EnsureOriginWithArg,
  },
  BoundedVec, WeakBoundedVec,
};
use frame_system::{pallet_prelude::*, Config as SystemConfig};
use sp_runtime::{
  traits::{MaybeDisplay, Saturating, StaticLookup, Zero},
  ArithmeticError, TokenError,
};
use sp_std::{fmt::Debug, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
  use super::*;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// The origin which may create or destroy an asset and acts as owner or the asset.
    /// Only organization member can crete an asset
    type CreateOrigin: EnsureOriginWithArg<Self::Origin, Self::AccountId>;

    /// The organization account identifier.
    type OrganizationId: Parameter
      + Member
      + MaybeSerializeDeserialize
      + Debug
      + MaybeDisplay
      + Ord
      + MaxEncodedLen;

    /// The maximum length of an asset's name stored on-chain.
    #[pallet::constant]
    type NameLimit: Get<u32>;

    /// The maximum number of topupped assets that the pallet can hold.
    #[pallet::constant]
    type MaxTopUppedAssets: Get<u32>;

    // The maximum count of fungible asset for each owner
    // #[pallet::constant]
    // type MaxAssets: Get<u32>;
  }

  #[pallet::storage]
  /// Details of an asset.
  pub(super) type Assets<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    AssetId,
    AssetDetails<T::AccountId, BoundedVec<u8, T::NameLimit>>,
  >;

  #[pallet::storage]
  /// Asset ids by owners (organizations).
  pub(super) type AssetsOf<T: Config> =
    StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, AssetId, ()>;

  #[pallet::storage]
  /// The holdings of a specific account for a specific asset
  pub(super) type Accounts<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    AssetId,
    AssetAccount,
    // OptionQuery,
    // GetDefault,
    // ConstU32<300_000>,
  >;

  #[pallet::storage]
  /// Storing next asset id
  pub type NextAssetId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

  #[pallet::storage]
  /// Storing assets which marked as Top Upped
  pub(super) type TopUppedAssets<T: Config> =
    StorageValue<_, WeakBoundedVec<AssetId, T::MaxTopUppedAssets>, ValueQuery>;

  #[pallet::storage]
  /// Accounts with assets which maybe need to top upped in next block.
  pub(super) type TopUpQueue<T: Config> =
    StorageDoubleMap<_, Blake2_128Concat, AssetId, Blake2_128Concat, T::AccountId, ()>;

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The asset has been created.
    Created {
      asset_id: AssetId,
      owner: T::AccountId,
    },
    /// Some assets were issued.
    Issued {
      asset_id: AssetId,
      owner: T::AccountId,
      total_supply: FungibleAssetBalance,
    },
    /// Some assets were destroyed.
    Burned {
      asset_id: AssetId,
      owner: T::AccountId,
      balance: FungibleAssetBalance,
    },
    /// Event documentation should end with an array that provides descriptive names for event
    /// parameters. [something, who]
    SomethingStored(u32, T::AccountId),
    /// An asset class was destroyed.
    Destroyed {
      asset_id: AssetId,
      owner: T::AccountId,
    },
  }

  #[pallet::error]
  pub enum Error<T> {
    /// Error names should be descriptive.
    NoneValue,
    /// Errors should have helpful documentation associated with them.
    StorageOverflow,
    /// The asset ID is already taken.
    InUse,
    // No available fungible asset id.
    NoAvailableAssetId,
    /// The signing account has no permission to do the operation.
    NoPermission,
    /// Asset name is too long.
    AssetNameTooLong,
    /// Limit of tipupped assets is reached.
    MaxTopUppedAssetsReached,
    /// Global Cup must be above zero.
    ZeroGlobalCup,
    /// Local Cup must be above zero.
    ZeroLocalCup,
    /// Top upped speed must be above zero.
    ZeroTopUpped,
    /// Top upped speed can't be set without a local cup.
    TopUppedWithNoCup,
    /// The account to alter does not exist.
    NoAccount,
  }

  // Implement the pallet hooks.
  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
      // All assets which need to be top upped (which stoded in TopUpQueue) must be processed
      let weight = Self::process_top_upped_assets();
      log::info!("💫 Top up processed [weigth: {}]", &weight);

      weight
    }

    // can implement also: on_finalize, on_runtime_upgrade, offchain_worker, ...
    // see `Hooks` trait
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Issue a new fungible asset from.
    ///
    /// This new asset has owner as orgaization.
    ///
    /// The origin must be Signed.
    ///
    ///
    /// Parameters:
    /// - `organization_id`: The identifier of the organization. Origin must be member of it.
    ///
    /// Emits `Created` event when successful.
    #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
    pub fn create(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      name: Vec<u8>,
      top_upped: Option<TopUppedFA>,
      cup_global: Option<CupFA>,
      cup_local: Option<CupFA>,
    ) -> DispatchResult {
      // owner of an asset wiil be orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only organization can create an asset
      T::CreateOrigin::ensure_origin(origin, &owner)?;

      let new_asset_details = AssetDetailsBuilder::<T>::new(owner.clone(), name)?
        .top_upped(top_upped)?
        .cup_global(cup_global)?
        .cup_local(cup_local)?
        .build()?;

      let asset_id = Self::get_next_asset_id()?;

      Assets::<T>::insert(asset_id, new_asset_details);
      // let mut asset_ids = Vec::<AssetId>::new();
      // asset_ids.push(asset_id);
      // let bounded_ids:BoundedVec<AssetId, T::MaxAssets> =
      // asset_ids.clone().try_into().expect("exceed allowed length");
      AssetsOf::<T>::insert(&owner, asset_id, ());
      // if asset is top upped, add it to top_upped_assets
      if let Some(top_upped) = top_upped {
        if top_upped.speed > Zero::zero() {
          Self::top_upped_asset_add(&asset_id).unwrap();
        }
      }

      Self::deposit_event(Event::Created { asset_id, owner });

      Ok(())
    }

    /// Destroy a fungible asset.
    ///
    /// The origin must be Signed and must be a member of the organization
    #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
    pub fn destroy(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      #[pallet::compact] asset_id: AssetId,
    ) -> DispatchResult {
      // owner of an asset wiil be orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only member if the organization can create an asset
      T::CreateOrigin::ensure_origin(origin, &owner)?;
      // TODO: set limits on the number of assets created by each organization
      Assets::<T>::remove(asset_id);
      AssetsOf::<T>::remove(&owner, asset_id);
      Self::top_upped_asset_remove(&asset_id);

      Self::deposit_event(Event::Destroyed { asset_id, owner });
      Ok(())
    }
  }
}
