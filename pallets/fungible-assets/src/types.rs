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
pub struct AssetDetails<AccountId, Balance> {
  pub(super) owner: AccountId,
  /// The total supply across all accounts.
	pub(super) supply: Balance,
}

#[derive(Default)]
pub struct AssetDetailsBuilder<T: Config> {
    owner: T::AccountId,
}

impl<T: pallet::Config> AssetDetailsBuilder<T> {
  pub fn new(owner: T::AccountId) -> AssetDetailsBuilder<T> {
    AssetDetailsBuilder {
      owner,
    }
  }

  pub fn build(self) -> AssetDetails<T::AccountId, T::Balance> {
    AssetDetails {
      owner: self.owner,
      supply: Zero::zero(),
    }
  }
  
}

/// Type of the fungible asset's ids
pub type AssetId = u32;
