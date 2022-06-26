#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_std::prelude::*;

mod types;
mod functions;
pub use types::*;

use codec::HasCompact;

use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
		MaybeDisplay, One,
	},
	ArithmeticError, TokenError, DispatchError,
};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*, vec::Vec};
use frame_support::{
	traits::{
		tokens::{fungibles, DepositConsequence, WithdrawConsequence},
		EnsureOriginWithArg,
		ReservableCurrency,
		Currency,
	},
	BoundedVec, log,
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
		/// Only organization member can crete an asset
		type CreateOrigin: EnsureOriginWithArg<Self::Origin, Self::AccountId>;

		/// The organization account identifier.
		type OrganizationId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaybeDisplay
			+ Ord
			+ MaxEncodedLen;
		
		/// The maximum length of an asset's name stored on-chain.
		#[pallet::constant]
		type NameLimit: Get<u32>;
		
		// The maximum count of fungible asset for each owner
		// #[pallet::constant]
		// type MaxAssets: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// #[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	/// Details of an asset.
	pub(super) type Assets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AssetId,
		AssetDetails<T::AccountId, T::Balance, BoundedVec<u8, T::NameLimit>>
	>;

	#[pallet::storage]
	#[pallet::getter(fn assets_of)]
	/// Asset ids by owners (organizations).
	pub(super) type AssetsOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		AssetId,
		(),
	>;

	#[pallet::storage]
	/// The holdings of a specific account for a specific asset
	pub(super) type Accounts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetId,
		Blake2_128Concat,
		T::AccountId,
		AssetAccountOf<T>,
		// OptionQuery,
		// GetDefault,
		// ConstU32<300_000>,
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
		/// The asset has been created.
		Created { asset_id: AssetId, owner: T::AccountId },
		/// Some assets were issued.
		Issued { asset_id: AssetId, owner: T::AccountId, total_supply: T::Balance },
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// An asset class was destroyed.
		Destroyed { asset_id: AssetId, owner: T::AccountId },
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
		/// Asset name is too long.
		AssetNameTooLong,
		/// Global Cup must be above zero.
		ZeroGlobalCup,
		/// Local Cup must be above zero.
		ZeroLocalCup,
		/// Top upped speed must be above zero.
		ZeroTopUpped,
		/// Top upped speed can't be set without a local cup.
		TopUppedWithNoCup,
	}

	// Implement the pallet hooks.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			log::info!(target: "runtime::fungible-assets", "hook called");
			// unimplemented!();
			T::DbWeight::get().reads_writes(1, 1)
		}

		// can implement also: on_finalize, on_runtime_upgrade, offchain_worker, ...
		// see `Hooks` trait
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
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
		pub fn create(
			origin: OriginFor<T>,
			organization_id: <T::Lookup as StaticLookup>::Source,
			name: Vec<u8>,
			top_upped: Option<TopUppedFA>,
			cup_global: Option<CupFA>,
			cup_local: Option<CupFA>,
		) -> DispatchResult {

			// owner of an asset wiil be orgnization
			let owner = T::Lookup::lookup(organization_id)?;
			// Only organization can create an asset
			T::CreateOrigin::ensure_origin(origin, &owner)?;

			let new_asset_details = AssetDetailsBuilder::<T>::new(owner.clone(), name)?
				.top_upped(top_upped)?
				.cup_global(cup_global)?
				.cup_local(cup_local)?
				.build()?;

			let asset_id = Self::get_next_asset_id()?;

			Assets::<T>::insert(
				asset_id.clone(),
				new_asset_details
			);
			// let mut asset_ids = Vec::<AssetId>::new();
			// asset_ids.push(asset_id);
			// let bounded_ids:BoundedVec<AssetId, T::MaxAssets> = asset_ids.clone().try_into().expect("exceed allowed length");
			AssetsOf::<T>::insert(
				&owner,
				&asset_id,
				()
			);

			Self::deposit_event(Event::Created { asset_id, owner });

			Ok(())
		}

		/// Destroy a fungible asset.
		/// 
		/// The origin must be Signed and must be a member of the organization
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
		pub fn destroy(origin: OriginFor<T>,
			organization_id: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] asset_id: AssetId,
		) -> DispatchResult {
			// owner of an asset wiil be orgnization
			let owner = T::Lookup::lookup(organization_id)?;
			// Only member if the organization can create an asset
			T::CreateOrigin::ensure_origin(origin, &owner)?;
			// TODO: set limits on the number of assets created by each organization
			Assets::<T>::remove(&asset_id);
			AssetsOf::<T>::remove(&owner, &asset_id);

			Self::deposit_event(Event::Destroyed { asset_id, owner });
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

