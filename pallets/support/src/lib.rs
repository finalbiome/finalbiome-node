//! Traits and associated utilities for use in the FinalBiome environment.
//! 
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;
// use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits:: {
		AtLeast32BitUnsigned,
	},
};

mod types;
mod constants;
pub use constants::*;
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
  pub value: AttributeValue,
}

/// An attribute data of the asset. \
/// Can be Number or String.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AttributeValue {
  Number(NumberAttribute),
  Text(BoundedVec<u8, AttributeValueStringLimit>)
}
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct NumberAttribute {
  pub number_value: u32,
  pub number_max: Option<u32>,
}

impl TryFrom<u32> for AttributeValue {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
      Ok(AttributeValue::Number(NumberAttribute {
        number_value: value,
        number_max: None,
      }))
    }
}
impl TryFrom<(u32, u32)> for AttributeValue {
    type Error = &'static str;

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
      if value.0 > value.1 {
        Err(ERROR_VALIDATE_NUMBER_ATTRIBUTE)
      } else {
        Ok(AttributeValue::Number(NumberAttribute {
          number_value: value.0,
          number_max: Some(value.1),
        }))
      }
    }
}
impl TryFrom<(u32, Option<u32>)> for AttributeValue {
    type Error = &'static str;

    fn try_from(value: (u32, Option<u32>)) -> Result<Self, Self::Error> {
      match value.1 {
        Some(max_val) => Self::try_from((value.0, max_val)),
        None => Self::try_from(value.0)
      }
    }
}
impl TryFrom<&str> for AttributeValue {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
      match value.as_bytes().to_vec().try_into() {
        Ok(val) => Ok(AttributeValue::Text(val)),
        Err(_) => Err(ERROR_VALIDATE_TEXT_ATTRIBUTE)
      }
      
    }
}
impl TryFrom<Vec<u8>> for AttributeValue {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
      match value.try_into() {
        Ok(val) => Ok(AttributeValue::Text(val)),
        Err(_) => Err(ERROR_VALIDATE_TEXT_ATTRIBUTE)
      }
      
    }
}

impl AttributeValue {
  pub fn validate(&self) -> DispatchResult {
    if let AttributeValue::Number(value) = self {
      if let Some(max_val) = value.number_max {
        if value.number_value > max_val {
          return Err(DispatchError::Other(ERROR_VALIDATE_NUMBER_ATTRIBUTE));
        }
      }
    };
    Ok(())
  }
}

impl Attribute {
  pub fn validate(&self) -> DispatchResult {
    self.value.validate()
  }
}
/// Type of the attribute key for NFA
pub type AttributeKey = BoundedVec<u8, AttributeKeyStringLimit>;
/// Represent a list of the attributes
pub type AttributeList = BoundedVec<Attribute, AttributeListLengthLimit>;
