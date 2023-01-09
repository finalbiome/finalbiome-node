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
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  use frame_support::traits::Currency;

use crate::AccountIdLookupOf;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    /// How often is the recovery of the number of tokens. In v1 unchanged, and equal to 24 hours in blocks.
    type RecoveryPeriod: Get<Self::BlockNumber>;
    /// The type for recording an account's balance.
	  type Currency: Currency<Self::AccountId>;
    /// The max capacity of the utility tokens for each user account.
    type Capacity: Get<<<Self as Config>::Currency as frame_support::traits::Currency<<Self as frame_system::Config>::AccountId>>::Balance>;
    /// How much slots can be exist in the storage. In v1 unchanged, and equal to RecoveryPeriod in blocks.
    type NumberOfSlots: Get<Self::BlockNumber>;
    /// Max number of accounts which can hold one slot.
    type AccountsInSlotLimit: Get<u32>;
  }

  // The pallet's runtime storage items.
  // // https://docs.substrate.io/main-docs/build/runtime-storage/
  #[pallet::storage]
  #[pallet::getter(fn something)]
  // Learn more about declaring storage items:
  // // https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
  pub type Something<T> = StorageValue<_, u32>;

  /// The `AccountId` of the Registrar key.
	#[pallet::storage]
	#[pallet::getter(fn registrar_key)]
	pub(super) type RegistrarKey<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

  /// Events of the Users pallet.
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event documentation should end with an array that provides descriptive names for event
    /// parameters. [something, who]
    SomethingStored(u32, T::AccountId),
    /// The \[Registrar\] just switched identity; the old key is supplied if one existed.
		KeyChanged { old_registrar: Option<T::AccountId> },
  }

  /// Errors of the Users pallet.
  #[pallet::error]
  pub enum Error<T> {
    /// Error names should be descriptive.
    NoneValue,
    /// Errors should have helpful documentation associated with them.
    StorageOverflow,
    /// Sender must be the Registrar account
    RequireRegistrar,
  }

  #[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The `AccountId` of the registrar key.
		pub registrar_key: Option<T::AccountId>,
	}

  #[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { registrar_key: None }
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
      // https://docs.substrate.io/main-docs/build/origins/
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
  
    /// Authenticates the current registrar key and sets the given AccountId (`new`) as the new registrar
		/// key.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - One DB read.
		/// - One DB change.
		/// # </weight>
    #[pallet::weight(0)]
    pub fn set_registrar_key(origin: OriginFor<T>, new: AccountIdLookupOf<T>) -> DispatchResultWithPostInfo {
      // This is a public call, so we ensure that the origin is some signed account.
      let sender = ensure_signed(origin)?;
      Self::ensure_registrar(sender)?;

      let new = T::Lookup::lookup(new)?;
      Self::deposit_event(Event::KeyChanged { old_registrar: RegistrarKey::<T>::get() });
			RegistrarKey::<T>::put(&new);

      // Registrar user does not pay a fee.
			Ok(Pays::No.into())
    }
  }
}
