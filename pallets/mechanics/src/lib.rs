#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
mod functions;

pub use types::*;
pub use pallet_support::{
	AccountIdOf, MechanicId, Index, MechanicIdOf,
	LockResult,
};

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use sp_runtime::{
	traits:: {
		Saturating,
	},
};

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
		/// The origin which may execute mechanics.
		/// 
		/// Mechanics can only be executed by a regular user, neither the organization nor any of its members can execute mechanics
		type ExecuteOrigin: frame_support::traits::EnsureOrigin<Self::Origin, Success = Self::AccountId> ;
		/// The maximum list length to pass to mechanics.
		#[pallet::constant]
		type AssetsListLimit: Get<u32>;
		/// Life time of the mechanic in number of block. When `current_block + mechanic_lifetime` occurs, mechanics will be destroyed.
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
		(),
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

	#[pallet::storage]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The mechanics were done.
		Finished { id: T::Index, owner: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Execute mechanic `Buy NFA`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1))]
		pub fn exec_buy_nfa(origin: OriginFor<T>, class_id: NonFungibleClassId, offer_id: u32) -> DispatchResult {
			// Only a regular user can execute mechanic
			let who = T::ExecuteOrigin::ensure_origin(origin)?;
			// Generate mechanic id
			let mechanic_id = Self::get_mechanic_id(&who);
			Self::do_buy_nfa(&who, &class_id, &offer_id)?;
			Self::deposit_event(Event::Finished { id: mechanic_id.nonce, owner: mechanic_id.account_id });
			Ok(())
		}
	}
}
