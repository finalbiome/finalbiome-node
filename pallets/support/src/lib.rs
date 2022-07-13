//! Traits and associated utilities for use in the FinalBiome environment.
//! 
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
// use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits:: {
		AtLeast32BitUnsigned,
	},
};
use frame_support::traits::tokens::WithdrawConsequence;

mod types;

#[cfg(test)]
mod tests;

/// Trait to collect together properties for a Fungible AssetsId.
pub trait AssetId: Member
  + Parameter
  + AtLeast32BitUnsigned
  + Default
  + Copy
  + MaybeSerializeDeserialize
  + MaxEncodedLen
  + TypeInfo {}

impl<T: Member
  + Parameter
  + AtLeast32BitUnsigned
  + Default
  + Copy
  + MaybeSerializeDeserialize
  + MaxEncodedLen
  + TypeInfo> AssetId
for T {}

/// Trait to collect together properties for a Fungible Assets Balance.
pub trait Balance: Member
  + Parameter
  + AtLeast32BitUnsigned
  + Default
  + Copy
  + MaybeSerializeDeserialize
  + MaxEncodedLen
  + TypeInfo
{}

impl<T: Member
  + Parameter
  + AtLeast32BitUnsigned
  + Default
  + Copy
  + MaybeSerializeDeserialize
  + MaxEncodedLen
  + TypeInfo> Balance
for T {}

/// Type of the fungible asset id
pub type FungibleAssetId = u32;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance = u128;
/// Type of the non-fungible asset id
pub type NonFungibleClassId = u32;
/// The units in which we record balances of the fungible assets
pub type NonFungibleAssetId = u32;

/// Trait for providing an interface to a fungible assets instances.
pub trait FungibleAssets<AccountId> {
  /// Returns `Failed` if the asset `balance` of `who` may not be decreased by `amount`, otherwise the consequence.
  fn can_withdraw(
		asset: FungibleAssetId,
		who: &AccountId,
		amount: FungibleAssetBalance,
	) -> WithdrawConsequence<FungibleAssetBalance>;
  /// Attempt to reduce the asset balance of who by amount.  \
  /// If not possible then don’t do anything. Possible reasons for failure include: \
  /// * Less funds in the account than amount
  /// * Liquidity requirements (locks, reservations) prevent the funds from being removed
  /// * Operation would require destroying the account and it is required to stay alive (e.g. because it’s providing a needed provider reference).
  /// 
  /// If successful it will reduce the overall supply of the underlying token.
  fn burn_from(
    asset: FungibleAssetId, 
    who: &AccountId, 
    amount: FungibleAssetBalance
  ) -> DispatchResultAs<FungibleAssetBalance>;
}

/// Trait for providing an interface to a non-fungible assets instances.
pub trait NonFungibleAssets<AccountId> {

  fn mint_into(
    class_id: &NonFungibleClassId,
    who: &AccountId
  ) -> DispatchResult;

  /// Returns offer by given id
  fn get_offer(
    class_id: &NonFungibleClassId,
    offer_id: &u32,
  ) -> DispatchResultAs<(FungibleAssetId, FungibleAssetBalance)>;
}

pub type DispatchResultAs<T> = sp_std::result::Result<T, sp_runtime::DispatchError>;
/// Type alias for `frame_system`'s account id. \
/// The user account identifier type for the runtime.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
