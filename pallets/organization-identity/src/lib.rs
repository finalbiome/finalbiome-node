#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
  pallet_prelude::*,
  traits::{EnsureOrigin, EnsureOriginWithArg},
};
use frame_system::{pallet_prelude::*, RawOrigin};
use sp_std::vec::Vec;

mod types;
pub use types::*;

mod functions;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::error::BadOrigin;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// Connector to fungible assets instances.
    type FungibleAssets: pallet_support::traits::FungibleAssets<Self::AccountId>;
    /// Connector to non-fungible assets instances.
    type NonFungibleAssets: pallet_support::traits::NonFungibleAssets<Self::AccountId, Self::Index>;
    /// The origin which may onboard to the game.
    ///
    /// Onboarding can only be used by a regular user, neither the organization nor any of its
    /// members can onboard into the gamse
    type ExecuteOrigin: frame_support::traits::EnsureOrigin<Self::Origin, Success = Self::AccountId>;
    /// The maximum length of an organization's name stored on-chain.
    #[pallet::constant]
    type StringLimit: Get<u32>;
    /// The maximum members per organization.
    #[pallet::constant]
    type MaxMembers: Get<u8>;
  }

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
    OrganizationIdOf<T>, // account id of the organization
    Blake2_128Concat,
    T::AccountId, // account id of the member
    (),
  >;

  #[pallet::storage]
  /// Users of organizations.
  ///
  /// Stores users who has been onboarded into the game
  pub(super) type UsersOf<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    OrganizationIdOf<T>, // account id of the organization
    Blake2_128Concat,
    T::AccountId, // account id of the user
    (),
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
    /// An asset class has been updated.
    UpdatedOrganization(OrganizationIdOf<T>),
    /// An member was added to an organization. [organization, member]
    MemberAdded(OrganizationIdOf<T>, T::AccountId),
    /// An member was removed from organization. [organization, member]
    MemberRemoved(OrganizationIdOf<T>, T::AccountId),
    /// Assets for the game has been airdropped.
    Onboard(OrganizationIdOf<T>, T::AccountId),
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
    /// Member not exits.
    NotMember,
    /// Account has already been onboarded.
    AlreadyOnboarded,
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
      ensure!(Self::is_organization(&org), Error::<T>::NotOrganization);

      let member_count = MemberCount::<T>::get(&org);

      // check that the number of members in the organization does not exceed the limit
      ensure!(
        member_count < T::MaxMembers::get(),
        Error::<T>::MembershipLimitReached
      );

      // organizations can't be a member
      ensure!(!Self::is_organization(&who), Error::<T>::InvalidMember);

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
      ensure!(Self::is_organization(&org), Error::<T>::NotOrganization);
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

    /// Set assets which will be airdroped at game onboarding
    #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,2))]
    pub fn set_onboarding_assets(
      origin: OriginFor<T>,
      organization_id: T::AccountId,
      assets: OnboardingAssets,
    ) -> DispatchResultWithPostInfo {
      let member = ensure_signed(origin)?;
      // only member can update an organization
      ensure!(
        MembersOf::<T>::contains_key(&organization_id, &member),
        Error::<T>::NotMember
      );

      Self::do_set_onboarding_assets(&organization_id, assets)?;

      Self::deposit_event(Event::UpdatedOrganization(organization_id));
      Ok(().into())
    }

    /// Onboirding to game
    #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,2))]
    pub fn onboarding(
      origin: OriginFor<T>,
      organization_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      // Only a regular user can execute mechanic
      let account = T::ExecuteOrigin::ensure_origin(origin)?;
      // Neither the organization nor the member can't onboard to the game
      ensure!(!Self::is_member_or_organization(&account), BadOrigin);
      // Users who are already onboarded are not allowed
      ensure!(
        !Self::is_user_of_organization(&account, &organization_id),
        Error::<T>::AlreadyOnboarded
      );

      Self::do_onboarding(&organization_id, &account)?;

      Self::deposit_event(Event::Onboard(organization_id, account));

      Ok(().into())
    }
  }
}

impl<T: Config> Pallet<T> {
  /// Returns true if account is an organization
  fn is_organization(account: &T::AccountId) -> bool {
    Organizations::<T>::contains_key(account)
  }
  /// Returns true if account is a member of any organization or organization
  fn is_member_or_organization(account: &T::AccountId) -> bool {
    Members::<T>::contains_key(account)
  }
  /// Returns true if account is a user of given organization
  fn is_user_of_organization(account: &T::AccountId, organization: &T::AccountId) -> bool {
    UsersOf::<T>::contains_key(organization, account)
  }
}

/// Ensures that an organization is invoking a dispatch.
pub struct EnsureOrganization<T: Config>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<T::Origin> for EnsureOrganization<T> {
  type Success = T::AccountId;

  fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
    o.into().and_then(|o| match o {
      RawOrigin::Signed(ref who) if <Pallet<T>>::is_organization(who) => Ok(who.clone()),
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
impl<T: Config> EnsureOriginWithArg<T::Origin, OrganizationIdOf<T>>
  for EnsureMemberOfOrganization<T>
{
  type Success = T::AccountId;

  fn try_origin(o: T::Origin, a: &OrganizationIdOf<T>) -> Result<Self::Success, T::Origin> {
    o.into().and_then(|o| match o {
      RawOrigin::Signed(ref who) if MembersOf::<T>::contains_key(a, who) => Ok(who.clone()),
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
      RawOrigin::Signed(ref who) if !<Pallet<T>>::is_member_or_organization(who) => Ok(who.clone()),
      r => Err(T::Origin::from(r)),
    })
  }

  #[cfg(feature = "runtime-benchmarks")]
  fn successful_origin() -> T::Origin {
    T::Origin::from(RawOrigin::Signed(Default::default()))
  }
}
