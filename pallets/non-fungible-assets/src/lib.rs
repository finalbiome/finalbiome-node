#![cfg_attr(not(feature = "std"), no_std)]

mod types;
mod functions;
mod impl_non_fubgible_assets;
mod characteristics;

pub use types::*;

pub use support;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_runtime::{
	traits:: {
		One, Zero,
		StaticLookup,
		Saturating,
	},
	DispatchError, ArithmeticError,
};
use sp_std::{vec::Vec};
use frame_support:: {
	traits:: {
		EnsureOriginWithArg,
	},
	WeakBoundedVec,
};

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The maximum length of an class name stored on-chain.
		#[pallet::constant]
		type ClassNameLimit: Get<u32>;
		/// The origin which may create or destroy a class and acts as owner or the class.
		/// Only organization member can crete a class
		type CreateOrigin: EnsureOriginWithArg<Self::Origin, Self::AccountId>;
		/// Connector to fungible assets instances
		type FungibleAssets: support::FungibleAssets;
		/// Lenght limit of the name for the bettor ouncome
		#[pallet::constant]
		type BettorOutcomeNameLimit: Get<u32>;
		/// Lenght limit of the string attribute value
		#[pallet::constant]
		type AttributeValueLimit: Get<u32>;
		/// The maximum length of an attribute key.
		#[pallet::constant]
		type AttributeKeyLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// Details of asset classes.
	pub(super) type Classes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NonFungibleClassId,
		ClassDetailsOf<T>,
	>;

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
		AssetDetails<T::AccountId>,
		OptionQuery,
	>;

	#[pallet::storage]
	/// Attributes of an asset class.
	pub(super) type Attributes<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, NonFungibleClassId>,
			NMapKey<Blake2_128Concat, Option<NonFungibleAssetId>>,
			NMapKey<Blake2_128Concat, BoundedVec<u8, T::AttributeKeyLimit>>,
		),
		AttributeDetails<StringAttribute<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	/// Storing the next asset id
	pub type NextAssetId<T: Config> = StorageValue<_, NonFungibleAssetId, ValueQuery>;

	#[pallet::storage]
	/// Storing the next class id
	pub type NextClassId<T: Config> = StorageValue<_, NonFungibleClassId, ValueQuery>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An asset class has been created.
		Created { class_id: NonFungibleClassId, owner: T::AccountId },
		/// An asset class has been destroyed.
		Destroyed { class_id: NonFungibleClassId },
		/// An asset `instance` has been issued.
		Issued { class_id: NonFungibleClassId, asset_id: NonFungibleAssetId, owner: T::AccountId },
		/// New attribute metadata has been set for the asset class.
		AttributeCreated {
			class_id: NonFungibleClassId, key: BoundedVec<u8, T::AttributeKeyLimit>, value: AttributeDetails<StringAttribute<T>> },
		/// Attribute metadata has been removed for the asset class.
		AttributeRemoved { class_id: NonFungibleClassId, key: BoundedVec<u8, T::AttributeKeyLimit> },
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
		/// The given asset ID is unknown.
		UnknownClass,
		/// The bettor characteristic is wrong.
		WrongBettor,
		/// Attribute value not supported
		AttributeConversionError,
		/// Attribute numeric value exceeds maximum value
		NumberAttributeValueExceedsMaximum,
		/// String attribute length limit exceeded
		StringAttributeLengthLimitExceeded,
		/// An attribute with the specified name already exists
		AttributeAlreadyExists,
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
			let class_details = ClassDetailsBuilder::<T>::new(owner.clone(), name)?
				.build()?;
			let class_id = Self::get_next_class_id()?;

			Classes::<T>::insert(
				class_id,
				class_details
			);
			ClassAccounts::<T>::insert(
				&owner,
				&class_id,
				()
			);

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
			attribute_name: Vec<u8>,
			attribute_value: AttributeTypeRaw,
		) -> DispatchResult {
			// owner of a class must be an orgnization
			let owner = T::Lookup::lookup(organization_id)?;
			// Only organization can manage an asset class
			T::CreateOrigin::ensure_origin(origin, &owner)?;

			Self::do_create_attribute(class_id, Some(owner), attribute_name, attribute_value)?;

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
			attribute_name: Vec<u8>,
		) -> DispatchResult {
			// owner of a class must be an orgnization
			let owner = T::Lookup::lookup(organization_id)?;
			// Only organization can manage an asset class
			T::CreateOrigin::ensure_origin(origin, &owner)?;

			Self::do_remove_attribute(class_id, Some(owner), attribute_name)?;
			
			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
