#![cfg_attr(not(feature = "std"), no_std)]

mod functions;
mod impl_non_fubgible_assets;
mod types;

pub use types::*;

pub use pallet_support::{
  self, purchased,
  types_nfa::{AssetDetails, ClassDetails},
  AccountIdOf, Attribute, AttributeKey, AttributeList, AttributeValue, Characteristic,
  DispatchResultAs, IndexOf, LockResultOf, NumberAttribute,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::EnsureOriginWithArg;
use sp_runtime::{
  traits::{Saturating, StaticLookup, Zero},
  ArithmeticError,
};
use sp_std::vec::Vec;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use pallet_support::{CommonError, Index};

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config<Index = Index> {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// The origin which may create or destroy a class and acts as owner or the class.
    /// Only organization member can crete a class
    type CreateOrigin: EnsureOriginWithArg<Self::Origin, Self::AccountId>;
    /// Connector to fungible assets instances
    type FungibleAssets: pallet_support::traits::FungibleAssets<Self::AccountId>;
  }

  #[pallet::storage]
  /// Details of asset classes.
  pub(super) type Classes<T: Config> =
    StorageMap<_, Blake2_128Concat, NonFungibleClassId, ClassDetailsOf<T>>;

  #[pallet::storage]
  /// The classes owned by any given account.
  pub(super) type ClassAccounts<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    NonFungibleClassId,
    (),
    OptionQuery,
  >;

  #[pallet::storage]
  /// The assets held by any given account.
  pub(super) type Accounts<T: Config> = StorageNMap<
    _,
    (
      NMapKey<Blake2_128Concat, T::AccountId>,
      NMapKey<Blake2_128Concat, NonFungibleClassId>,
      NMapKey<Blake2_128Concat, NonFungibleAssetId>,
    ),
    (),
    OptionQuery,
  >;

  #[pallet::storage]
  /// Details of assets.
  pub(super) type Assets<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    NonFungibleClassId,
    Blake2_128Concat,
    NonFungibleAssetId,
    AssetDetails<T::AccountId, T::Index>,
    OptionQuery,
  >;

  #[pallet::storage]
  /// Attributes of an assets.
  pub(super) type Attributes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    NonFungibleAssetId,
    Blake2_128Concat,
    AttributeKey,
    AttributeValue,
    OptionQuery,
  >;

  #[pallet::storage]
  /// Attributes of an asset class.
  pub(super) type ClassAttributes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    NonFungibleClassId,
    Blake2_128Concat,
    AttributeKey,
    AttributeValue,
    OptionQuery,
  >;

  #[pallet::storage]
  /// Storing the next asset id
  pub type NextAssetId<T: Config> = StorageValue<_, NonFungibleAssetId, ValueQuery>;

  #[pallet::storage]
  /// Storing the next class id
  pub type NextClassId<T: Config> = StorageValue<_, NonFungibleClassId, ValueQuery>;

  // Pallets use events to inform users when important changes are made.
  // https://docs.substrate.io/v3/runtime/events-and-errors
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// An asset class has been created.
    Created {
      class_id: NonFungibleClassId,
      owner: T::AccountId,
    },
    /// An asset class has been destroyed.
    Destroyed { class_id: NonFungibleClassId },
    /// An asset class has been updated.
    Updated { class_id: NonFungibleClassId },
    /// An asset `instance` has been issued.
    Issued {
      class_id: NonFungibleClassId,
      asset_id: NonFungibleAssetId,
      owner: T::AccountId,
    },
    /// New attribute metadata has been set for the asset class.
    AttributeCreated {
      class_id: NonFungibleClassId,
      key: AttributeKey,
      value: AttributeValue,
    },
    /// Attribute metadata has been removed for the asset class.
    AttributeRemoved {
      class_id: NonFungibleClassId,
      key: AttributeKey,
    },
    /// An asset `instance` was destroyed.
    Burned {
      class_id: NonFungibleClassId,
      asset_id: NonFungibleAssetId,
      owner: T::AccountId,
    },
    /// Event documentation should end with an array that provides descriptive names for event
    /// parameters. [something, who]
    SomethingStored(u32, T::AccountId),
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Error names should be descriptive.
    NoneValue,
    /// Errors should have helpful documentation associated with them.
    StorageOverflow,
    // No available non-fungible asset id.
    NoAvailableAssetId,
    // No available non-fungible asset id.
    NoAvailableClassId,
    /// Class name is too long.
    ClassNameTooLong,
    /// The signing account has no permission to do the operation.
    NoPermission,
    /// The given class Id is unknown.
    UnknownClass,
    /// The given asset Id is unknown.
    UnknownAsset,
    /// The bettor characteristic is wrong.
    WrongBettor,
    /// The purchased characteristic is wrong.
    WrongPurchased,
    /// Attribute value not supported
    AttributeConversionError,
    /// Attribute numeric value exceeds maximum value
    NumberAttributeValueExceedsMaximum,
    /// String attribute length limit exceeded
    StringAttributeLengthLimitExceeded,
    /// An attribute with the specified name already exists
    AttributeAlreadyExists,
    /// General error if any parameter is invalid
    WrongParameter,
    /// This characteristic is not supported by this asset
    UnsupportedCharacteristic,
    /// Characteristic is invalid
    WrongCharacteristic,
    /// The asset instance is locked
    Locked,
    /// The common error
    CommonError(CommonError),
  }

  impl<T> From<CommonError> for Error<T> {
    fn from(t: CommonError) -> Self {
      match t {
        CommonError::WrongBettor => Error::<T>::WrongBettor,
        CommonError::WrongPurchased => Error::<T>::WrongPurchased,
        _ => Error::<T>::CommonError(t),
      }
    }
  }

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Issue a new non fungible class.
    ///
    /// This new class has owner as orgaization.
    ///
    /// The origin must be Signed.
    ///
    /// Parameters:
    /// - `organization_id`: The identifier of the organization. Origin must be member of it.
    ///
    /// Emits `Created` event when successful.
    #[pallet::weight(T::DbWeight::get().reads_writes(2, 3))]
    pub fn create(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      name: Vec<u8>,
    ) -> DispatchResult {
      // owner of a class must be an orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only organization can create an asset class
      T::CreateOrigin::ensure_origin(origin, &owner)?;
      let class_details = ClassDetailsBuilder::<T>::new(owner.clone(), name)?.build()?;
      let class_id = Self::get_next_class_id()?;

      Classes::<T>::insert(class_id, class_details);
      ClassAccounts::<T>::insert(&owner, class_id, ());

      Self::deposit_event(Event::Created { class_id, owner });

      Ok(())
    }

    /// Destroy a non fungible asset class.
    ///
    /// The origin must be Signed and must be a member of the organization
    #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
    pub fn destroy(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      #[pallet::compact] class_id: NonFungibleClassId,
    ) -> DispatchResult {
      // owner of an asset must be an organization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only member of the organization can destory an asset class
      T::CreateOrigin::ensure_origin(origin, &owner)?;
      // Organization must be an owner of the class
      Self::do_destroy_class(class_id, Some(owner))?;

      Ok(())
    }

    /// Creates an attribute for the non fungible asset class.
    /// The origin must be Signed, be a member of the organization
    /// and that organization must be a owner of the asset class.
    #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
    pub fn create_attribute(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      #[pallet::compact] class_id: NonFungibleClassId,
      attribute: Attribute,
    ) -> DispatchResult {
      // owner of a class must be an orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only organization can manage an asset class
      T::CreateOrigin::ensure_origin(origin, &owner)?;

      Self::do_create_attribute(class_id, Some(owner), attribute)?;

      Ok(())
    }

    /// Removes an attribute for the non fungible asset class.
    /// The origin must be Signed, be a member of the organization
    /// and that organization must be a owner of the asset class.
    #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
    pub fn remove_attribute(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      #[pallet::compact] class_id: NonFungibleClassId,
      attribute_name: AttributeKey,
    ) -> DispatchResult {
      // owner of a class must be an orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only organization can manage an asset class
      T::CreateOrigin::ensure_origin(origin, &owner)?;

      Self::do_remove_attribute(class_id, Some(owner), attribute_name)?;

      Ok(())
    }

    #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
    pub fn set_characteristic(
      origin: OriginFor<T>,
      organization_id: <T::Lookup as StaticLookup>::Source,
      #[pallet::compact] class_id: NonFungibleClassId,
      characteristic: Characteristic,
    ) -> DispatchResult {
      // owner of a class must be an orgnization
      let owner = T::Lookup::lookup(organization_id)?;
      // Only organization can manage an asset class
      T::CreateOrigin::ensure_origin(origin, &owner)?;

      Self::do_set_characteristic(class_id, Some(owner), characteristic)?;

      Ok(())
    }
  }
}
