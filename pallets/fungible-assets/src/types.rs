use super::*;
use frame_support::{
	pallet_prelude::*,
	traits::{fungible, tokens::BalanceConversion},
};
// use frame_system::Account;

// /// Identifier of the asset.
// pub(super) type AssetId = impl Member
//   + Parameter
//   + Default
//   + Copy
//   + HasCompact
//   + MaybeSerializeDeserialize
//   + MaxEncodedLen
//   + TypeInfo;

// type BalanceOf<F, T> = <F as fungible::Inspect<AccountIdOf<T>>>::Balance;
// pub type OrganizationIdOf<T> = <T as pallet::Config>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<AccountId, Balance, BoundedString> {
  pub(super) owner: AccountId,
  /// The total supply across all accounts.
	pub(super) supply: Balance,
  /// Name of the Asset. Limited in length by `NameLimit`.
	pub(super) name: BoundedString,
}

#[derive(Default)]
pub struct AssetDetailsBuilder<T: Config> {
    owner: T::AccountId,
    name: NameLimit<T>,
}

impl<T: pallet::Config> AssetDetailsBuilder<T> {
  pub fn new(owner: T::AccountId, name: NameLimit<T>) -> AssetDetailsBuilder<T> {
    AssetDetailsBuilder {
      owner,
      name,
    }
  }

  pub fn build(self) -> AssetDetails<T::AccountId, T::Balance, NameLimit<T>> {
    AssetDetails {
      owner: self.owner,
      supply: Zero::zero(),
      name: self.name,
    }
  }
  
}

/// Type of the fungible asset's ids
pub type AssetId = u32;

type NameLimit<T> = BoundedVec<u8, <T as pallet::Config>::NameLimit>;
