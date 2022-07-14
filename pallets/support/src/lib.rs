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

mod types;
mod constants;
use constants::*;
pub mod traits;

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

pub type DispatchResultAs<T> = sp_std::result::Result<T, sp_runtime::DispatchError>;
/// Type alias for `frame_system`'s account id. \
/// The user account identifier type for the runtime.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Represent a single attribute of NFA as a key and value
pub struct Attribute {
  pub key: AttributeKey,
  pub value: AttributeDetails,
}

/// An attribute data of the asset. \
/// Can be Number or String.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AttributeDetails {
  Number(NumberAttribute),
  String(BoundedVec<u8, AttributeValueStringLimit>)
}
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct NumberAttribute {
  pub number_value: u32,
  pub number_max: Option<u32>,
}

/// Type of the attribute key for NFA
pub type AttributeKey = BoundedVec<u8, AttributeKeyStringLimit>;
/// Represent a list of the attributes
pub type AttributeList = BoundedVec<Attribute, AttributeListLengthLimit>;
