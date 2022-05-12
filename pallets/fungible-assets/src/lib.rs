#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use types::*;

use codec::HasCompact;

use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
		MaybeDisplay, One,
	},
	ArithmeticError, TokenError, DispatchError,
};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		
		/// The origin which may create or destroy an asset and acts as owner or the asset.
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		/// The organization account identifier.
		type OrganizationId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaybeDisplay
			+ Ord
			+ MaxEncodedLen;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	/// Details of an asset.
	pub(super) type Assets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AssetId,
		AssetDetails<T::AccountId, T::Balance>
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	/// Storing next asset id
	pub type NextAssetId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

	
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Asset was created.
		Created { asset_id: AssetId, owner: T::AccountId },

		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
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
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create(
			origin: OriginFor<T>,
			organization_id: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			// let _ = ensure_signed(origin)?;
			T::CreateOrigin::ensure_origin(origin)?;
			//.ok_or(Errors::T::NoPermission);

			let owner = T::Lookup::lookup(organization_id)?;

			// ensure!(!Assets::<T>::contains_key(asset_id), Error::<T>::InUse);

			let new_asset_details = AssetDetailsBuilder::<T>::new(owner.clone()).build();
			let asset_id = Self::get_next_asset_id()?;
			Assets::<T>::insert(
				asset_id,
				new_asset_details
			);

			// ensure!(!min_balance.is_zero(), Error::<T, I>::MinBalanceZero);

			// let deposit = T::AssetDeposit::get();
			// T::Currency::reserve(&owner, deposit)?;

			Self::deposit_event(Event::Created { asset_id, owner });
			

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

impl<T: Config> Pallet<T> {
	/// Generate next id for new asset
	fn get_next_asset_id() -> Result<AssetId, DispatchError> {
		NextAssetId::<T>::try_mutate(|id| -> Result<AssetId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;
			Ok(current_id)
		})
	}
}
