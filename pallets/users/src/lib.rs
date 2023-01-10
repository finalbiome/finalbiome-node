#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_runtime::traits::StaticLookup;

mod functions;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{pallet_prelude::*, traits::Currency};
  use frame_system::pallet_prelude::*;

  use crate::AccountIdLookupOf;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// How often is the recovery of the number of tokens. In v1 unchanged, and equal to 24 hours in
    /// blocks.
    #[pallet::constant]
    type RecoveryPeriod: Get<Self::BlockNumber>;
    /// The type for recording an account's balance.
    type Currency: Currency<Self::AccountId>;
    /// The max capacity of the utility tokens for each user account.
    #[pallet::constant]
    type Capacity: Get<
      <<Self as Config>::Currency as frame_support::traits::Currency<
        <Self as frame_system::Config>::AccountId,
      >>::Balance,
    >;
    /// How much slots can be exist in the storage. In v1 unchanged, and equal to RecoveryPeriod in
    /// blocks.
    #[pallet::constant]
    type NumberOfSlots: Get<Self::BlockNumber>;
    /// Max number of accounts which can hold one slot.
    #[pallet::constant]
    type AccountsPerSlotLimit: Get<u32>;
  }

  /// The `AccountId` of the Registrar key.
  #[pallet::storage]
  #[pallet::getter(fn registrar_key)]
  pub(super) type RegistrarKey<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Accounts which quotas should be restored when will be active specific slot.
  #[pallet::storage]
  pub(super) type Slots<T: Config> = StorageMap<
    _,
    Twox64Concat,
    T::BlockNumber,
    BoundedVec<T::AccountId, T::AccountsPerSlotLimit>,
    ValueQuery,
  >;

  /// Lookup from an account to the slot number.
  #[pallet::storage]
  pub(super) type SlotsLookup<T: Config> =
    StorageMap<_, Twox64Concat, T::AccountId, T::BlockNumber, ValueQuery>;

  /// Events of the Users pallet.
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// The \[Registrar\] just switched identity; the old key is supplied if one existed.
    KeyChanged { old_registrar: Option<T::AccountId> },
  }

  /// Errors of the Users pallet.
  #[pallet::error]
  pub enum Error<T> {
    /// Sender must be the Registrar account
    RequireRegistrar,
    /// Exceeded account limit per slot.
    Exhausted,
    /// The account is already registered.
    Registered,
  }

  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// The `AccountId` of the registrar key.
    pub registrar_key: Option<T::AccountId>,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        registrar_key: None,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      if let Some(ref key) = self.registrar_key {
        RegistrarKey::<T>::put(key);
      }
    }
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// Restore quotas
    fn on_initialize(now: T::BlockNumber) -> Weight {
      let active_slot = Self::get_active_slot(now);
      Self::service_quotas(active_slot)
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Authenticates the current registrar key and sets the given AccountId (`new`) as the new
    /// registrar key.
    ///
    /// The dispatch origin for this call must be _Signed_.
    ///
    /// # <weight>
    /// - O(1).
    /// - One DB read.
    /// - One DB change.
    /// # </weight>
    #[pallet::weight(0)]
    pub fn set_registrar_key(
      origin: OriginFor<T>,
      new: AccountIdLookupOf<T>,
    ) -> DispatchResultWithPostInfo {
      // This is a public call, so we ensure that the origin is some signed account.
      let sender = ensure_signed(origin)?;
      Self::ensure_registrar(sender)?;

      let new = T::Lookup::lookup(new)?;
      Self::deposit_event(Event::KeyChanged {
        old_registrar: RegistrarKey::<T>::get(),
      });
      RegistrarKey::<T>::put(&new);

      // Registrar user does not pay a fee.
      Ok(Pays::No.into())
    }

    /// Register a new account.
    ///
    /// User can register only once.
    #[pallet::weight(0)]
    pub fn sign_up(origin: OriginFor<T>, who: AccountIdLookupOf<T>) -> DispatchResultWithPostInfo {
      // This is a public call, so we ensure that the origin is some signed account.
      let sender = ensure_signed(origin)?;
      // Only registrar can make this call
      Self::ensure_registrar(sender)?;

      let target = T::Lookup::lookup(who)?;
      Self::do_sign_up(target)?;

      // Registrar user does not pay a fee.
      Ok(Pays::No.into())
    }
  }
}
