#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_std::vec::Vec;

mod types;
pub use types::*;

pub use pallet::*;

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
		/// The maximum length of an organization's name stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// The maximum members per organization.
		#[pallet::constant]
		type MaxMembers: Get<u8>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	/// Details of an organization.
	pub(super) type Organizations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		OrganizationDetails<BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	/// Members of organizations.
	pub(super) type MemberOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		Blake2_128Concat,
		T::AccountId, // account id of the member
		()
	>;

	#[pallet::storage]
	/// Counts of members in organization.
	pub(super) type MemberCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		u8,
		ValueQuery,
	>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// An organization has been created. [organization_name, who]
		CreatedOrganization(Vec<u8>, T::AccountId),
		/// An account was added to an organization. [organization_name, member]
		AddedToOrganization(Vec<u8>, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Cannot create the organization because it already exists.
		OrganizationExists,
		/// Cannot add users to a non-existent organization.
		InvalidOrganization,
		/// Cannot add a user to an organization to which they already belong.
		AlreadyMember,
		/// Cannot add another member because the limit is already reached.
		MembershipLimitReached,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
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


		/// Create an organization.
		/// Will return an OrganizationExists error if the organization has already
		/// been created. Will emit a CreatedOrganization event on success.
		///
		/// The dispatch origin for this call must be Signed.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_organization(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let new_organization = ensure_signed(origin)?;

			// We don't want to add duplicate organizations, so we check whether the potential new
			// organization is already present in the list. Because the organization is stored as a hash
			// map this check is constant time O(1)
			ensure!(
				!Organizations::<T>::contains_key(&new_organization),
				Error::<T>::OrganizationExists
			);

			// let member_count = MemberCount::<T>::get(&new_organization);
			// ensure!(
			// 	member_count < T::MaxMembers::get(),
			// 	Error::<T>::MembershipLimitReached
			// );

			// Insert new organization and emit the event
			let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().expect("Organization name is too long");
			let new_org_details = OrganizationDetails::new(bounded_name);
			Organizations::<T>::insert(&new_organization, new_org_details);
			// Asset::<T, I>::try_mutate(id, |maybe_asset| {
			// 	let mut asset = maybe_asset.take().ok_or(Error::<T, I>::Unknown)?;
			// 	asset.owner = T::Lookup::lookup(owner)?;
			// 	asset.issuer = T::Lookup::lookup(issuer)?;
			// 	asset.admin = T::Lookup::lookup(admin)?;
			// 	asset.freezer = T::Lookup::lookup(freezer)?;
			// 	asset.min_balance = min_balance;
			// 	asset.is_sufficient = is_sufficient;
			// 	asset.is_frozen = is_frozen;
			// 	*maybe_asset = Some(asset);

			// 	Self::deposit_event(Event::AssetStatusChanged { asset_id: id });
			// 	Ok(())
			// })
			// MemberCount::<T>::put(member_count + 1); // overflow check not necessary because of maximum
			Self::deposit_event(Event::CreatedOrganization(name, new_organization));

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
