#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;
mod types;

pub use pallet_support::{
  AccountIdOf, ClassDetailsOf, Index, LockResult, MechanicId, MechanicIdOf,
};
pub use types::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use sp_runtime::traits::Saturating;

use frame_support::{log, traits::Randomness};

#[frame_support::pallet]
pub mod pallet {
  use super::*;

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config<Index = Index> {
    /// The runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// Connector to fungible assets instances.
    type FungibleAssets: pallet_support::traits::FungibleAssets<Self::AccountId>;
    /// Connector to non-fungible assets instances.
    type NonFungibleAssets: pallet_support::traits::NonFungibleAssets<Self::AccountId, Self::Index>;
    /// Something that provides randomness in the runtime.
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
    /// The origin which may execute mechanics.
    ///
    /// Mechanics can only be executed by a regular user, neither the organization nor any of its
    /// members can execute mechanics
    type ExecuteOrigin: frame_support::traits::EnsureOrigin<Self::Origin, Success = Self::AccountId>;
    /// The maximum list length to pass to mechanics.
    #[pallet::constant]
    type AssetsListLimit: Get<u32>;
    /// Life time of the mechanic in number of block. When `current_block + mechanic_lifetime`
    /// occurs, mechanics will be destroyed.
    #[pallet::constant]
    type MechanicsLifeTime: Get<Self::BlockNumber>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  /// Store of the Mechanics.
  pub(super) type Mechanics<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    T::Index,
    MechanicDetailsOf<T>,
    OptionQuery,
  >;

  #[pallet::storage]
  /// Schedule when mechanics time out
  pub(super) type Timeouts<T: Config> = StorageNMap<
    _,
    (
      NMapKey<Blake2_128Concat, T::BlockNumber>, // when time out will happen
      NMapKey<Blake2_128Concat, T::AccountId>,
      NMapKey<Blake2_128Concat, T::Index>,
    ),
    (),
    OptionQuery,
  >;

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Mechanics done.
    Finished {
      owner: T::AccountId,
      id: T::Index,
      result: EventMechanicResult,
    },
    /// Mechanics was stopped.
    Stopped {
      owner: T::AccountId,
      id: T::Index,
      reason: EventMechanicStopReason,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Mechanics are not available for this asset or this origin
    MechanicsNotAvailable,
    /// Internal error
    Internal,
    /// The number of assets exceeds the allowable
    AssetsExceedsAllowable,
    /// Asset is incompatible with mechanic
    IncompatibleAsset,
    /// Given data is incompatible with mechanic
    IncompatibleData,
    /// The signing account has no permission to do the operation.
    NoPermission,
  }

  // Implement the pallet hooks.
  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
      // All assets which need to be top upped (which stoded in TopUpQueue) must be processed
      let (weight, count) = Self::process_mechanic_timeouts();
      log::info!(
        "🧹 Timeouted mechanics dropped [mechanics: {}, weigth: {}]",
        &count,
        &weight
      );

      weight
    }

    // can implement also: on_finalize, on_runtime_upgrade, offchain_worker, ...
    // see `Hooks` trait
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Execute mechanic `Buy NFA`
    #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
    pub fn exec_buy_nfa(
      origin: OriginFor<T>,
      class_id: NonFungibleClassId,
      offer_id: u32,
    ) -> DispatchResult {
      // Only a regular user can execute mechanic
      let who = T::ExecuteOrigin::ensure_origin(origin)?;
      // Generate mechanic id
      let mechanic_id = Self::get_mechanic_id(&who);
      let asset_id = Self::do_buy_nfa(&who, &class_id, &offer_id)?;

      let result: EventMechanicResult = Some(EventMechanicResultData::BuyNfa(asset_id));
      Self::deposit_event(Event::Finished {
        id: mechanic_id.nonce,
        owner: mechanic_id.account_id,
        result,
      });
      Ok(())
    }

    /// Execute mechanic `Bet`
    #[pallet::weight(T::DbWeight::get().reads_writes(5, 5))]
    pub fn exec_bet(
      origin: OriginFor<T>,
      class_id: NonFungibleClassId,
      asset_id: NonFungibleAssetId,
    ) -> DispatchResult {
      // Only a regular user can execute mechanic
      let who = T::ExecuteOrigin::ensure_origin(origin)?;
      Self::do_bet(&who, &class_id, &asset_id)?;
      Ok(())
    }

    /// Upgrade mechanic
    #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
    pub fn upgrade(origin: OriginFor<T>, upgrage_data: MechanicUpgradeDataOf<T>) -> DispatchResult {
      // Only a regular user can upgrade mechanic
      let who = T::ExecuteOrigin::ensure_origin(origin)?;
      Self::do_upgrade(&who, upgrage_data)?;
      Ok(())
    }
  }
}
