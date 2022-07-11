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
use frame_support::{traits::{EnsureOrigin, EnsureOriginWithArg}};
use frame_system::RawOrigin;
use frame_system::pallet_prelude::*;

mod types;
pub use types::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

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

	#[pallet::storage]
	/// Details of an organization.
	pub(super) type Organizations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		OrganizationDetails<BoundedVec<u8, T::StringLimit>>,
	>;

	#[pallet::storage]
	/// Details of an members.
	/// ATTENTION: The store also includes organizations.
	pub(super) type Members<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, // account id of the member
		(),
	>;

	#[pallet::storage]
	/// Members of organizations.
	pub(super) type MembersOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		Blake2_128Concat,
		T::AccountId, // account id of the member
		()
	>;

	#[pallet::storage]
	#[pallet::getter(fn member_count)]
	/// Counts of members in organization.
	pub(super) type MemberCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OrganizationIdOf<T>, // account_id of the organization
		u8,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Genesis assets: account_id, name
		pub organizations: Vec<(OrganizationIdOf<T>, Vec<u8>)>,
		/// Genesis metadata: organization_id, account_id
		pub members_of: Vec<(OrganizationIdOf<T>, T::AccountId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				organizations: Default::default(),
				members_of: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (org_id, name) in &self.organizations {
				assert!(!Organizations::<T>::contains_key(&org_id), "Organization id already in use");
				Organizations::<T>::insert(
					&org_id,
					OrganizationDetails::new(name.clone().try_into().expect(Error::<T>::OrganizationNameTooLong.into())),
				);
				Members::<T>::insert(&org_id, ());
			}
			let members_limit = T::MaxMembers::get();
			for (org_id, member_id) in &self.members_of {
				assert!(Organizations::<T>::contains_key(&org_id), "Organization does not exist");
				assert!(!MembersOf::<T>::contains_key(&org_id, &member_id), "Member id already in organization");
				let member_count = MemberCount::<T>::get(&org_id);
				assert!(member_count < members_limit, "The maximum members per organization exceeded");
				Members::<T>::insert(&member_id, ());
				MembersOf::<T>::insert(
					&org_id,
					&member_id,
					()
				);
				MemberCount::<T>::insert(&org_id, member_count + 1);
			}
		}
	}


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
		/// An member was added to an organization. [organization, member]
		MemberAdded(OrganizationIdOf<T>, T::AccountId),
		/// An member was removed from organization. [organization, member]
		MemberRemoved(OrganizationIdOf<T>, T::AccountId)
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
		/// Organization name is too long.
		OrganizationNameTooLong,
		/// Account is not an organization
		NotOrganization,
		/// Cannot add a user to an organization to which they already belong.
		AlreadyMember,
		/// Cannot add another member because the limit is already reached.
		MembershipLimitReached,
		/// Cannot add organization as an organization's member.
		InvalidMember,
		/// Member not exits&
		NotMember,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an organization.
		/// Will return an OrganizationExists error if the organization has already
		/// been created. Will emit a CreatedOrganization event on success.
		///
		/// The dispatch origin for this call must be Signed.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_organization(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
			let new_organization = ensure_signed(origin)?;

			// We don't want to add duplicate organizations, so we check whether the potential new
			// organization is already present in the list. Because the organization is stored as a hash
			// map this check is constant time O(1)
			ensure!(
				!Organizations::<T>::contains_key(&new_organization),
				Error::<T>::OrganizationExists
			);

			

			// Insert new organization and emit the event
			let bounded_name = name.clone().try_into();
			let bounded_name = match bounded_name {
				Ok(name) => name,
				Err(_) => return Err(Error::<T>::OrganizationNameTooLong.into()),
			};
			let new_org_details = OrganizationDetails::new(bounded_name);
			Organizations::<T>::insert(&new_organization, new_org_details);
			// Add created organization as a member, for performance reasons
			Members::<T>::insert(&new_organization, ());
			
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
			
			Self::deposit_event(Event::CreatedOrganization(name, new_organization));

			Ok(().into())
		}

		/// Add member to an organization.
		/// 
		/// # Events
		/// * `MemberAdded`
		/// # Errors
		/// * `NotOrganization` if origin not an organization
		/// * `MembershipLimitReached` if members limit exceeded
		/// * `InvalidMember` if member is organization
		/// * `AlreadyMember` if member already added
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4,2))]
		pub fn add_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
			let org = ensure_signed(origin)?;
			// only organization's account can add members
			ensure!(
				Self::is_organization(&org),
				Error::<T>::NotOrganization
			);

			let member_count = MemberCount::<T>::get(&org);

			// check that the number of members in the organization does not exceed the limit
			ensure!(
				member_count < T::MaxMembers::get(),
				Error::<T>::MembershipLimitReached
			);

			// organizations can't be a member
			ensure!(
				!Self::is_organization(&who),
				Error::<T>::InvalidMember
			);
			

			// check that the who is not a member
			ensure!(
				!MembersOf::<T>::contains_key(&org, &who),
				Error::<T>::AlreadyMember
			);

			Members::<T>::insert(&who, ());
			MembersOf::<T>::insert(&org, &who, ());
			MemberCount::<T>::insert(&org, member_count + 1); // overflow check not necessary because of maximum
			
			Self::deposit_event(Event::MemberAdded(org, who));

			Ok(().into())
		}

		/// Removes a member from organization.
		/// 
		/// # Events
		/// * `MemberRemoved`
		/// 
		/// # Errors
		/// * `NotOrganization` if origin not an organization
		/// * `NotMember` if a member doesn't exist
		/// * ``
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,2))]
		pub fn remove_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
			let org = ensure_signed(origin)?;
			// only organization's account can removes members
			ensure!(
				Self::is_organization(&org),
				Error::<T>::NotOrganization
			);
			// ensure the member exists
			ensure!(
				MembersOf::<T>::contains_key(&org, &who),
				Error::<T>::NotMember
			);

			Members::<T>::remove(&who);
			MembersOf::<T>::remove(&org, &who);
			MemberCount::<T>::mutate(&org, |c| *c -= 1);

			Self::deposit_event(Event::MemberRemoved(org, who));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Returns true if account is an organization
	fn is_organization(account: &T::AccountId) -> bool {
		Organizations::<T>::contains_key(&account)
	}
	/// Returns true if account is a member of any organization or organization
	fn is_member_or_organization(account: &T::AccountId) -> bool {
		Members::<T>::contains_key(&account)
	}
}

/// Ensures that an organization is invoking a dispatch.
pub struct EnsureOrganization<T: Config>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrganization<T> {
	type Success = T::AccountId;

	fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(ref who)
				if  <Pallet<T>>::is_organization(who) => Ok(who.clone()),
				r => Err(T::Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> T::Origin {
		T::Origin::from(RawOrigin::Signed(Default::default()))
	}
}

/// Ensures that the origin is a member of given organization
pub struct EnsureMemberOfOrganization<T: Config>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOriginWithArg<T::Origin, OrganizationIdOf<T>> for EnsureMemberOfOrganization<T> {
	type Success = T::AccountId;

	fn try_origin(o: T::Origin, a: &OrganizationIdOf<T>) -> Result<Self::Success, T::Origin> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(ref who) if MembersOf::<T>::contains_key(&a, &who) => Ok(who.clone()),
			r => Err(T::Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin(_: &OrganizationIdOf<T>) -> T::Origin {
		T::Origin::from(RawOrigin::Signed(Default::default()))
	}
}

/// Ensures that neither the organization nor any member is invoking a dispatch.
pub struct EnsureUser<T: Config>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureUser<T> {
	type Success = T::AccountId;

	fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(ref who)
				if  !<Pallet<T>>::is_member_or_organization(who) => Ok(who.clone()),
				r => Err(T::Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> T::Origin {
		T::Origin::from(RawOrigin::Signed(Default::default()))
	}
}


