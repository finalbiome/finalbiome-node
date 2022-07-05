//! Traits and associated utilities for use in the FinalBiome environment.
//! 
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits:: {
		One, Zero,
		StaticLookup,
		AtLeast32BitUnsigned,
	},
	DispatchError,
};

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

/// Trait for providing an interface to a fungible assets instances.
pub trait FungibleAssets {
  /// Type of the FA id
  type AssetId: AssetId;
  /// The units in which records balances of FA.
  type Balance: Balance;
}
