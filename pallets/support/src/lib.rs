//! Traits and associated utilities for use in the FinalBiome environment.
//! 
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::inherent::Vec;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::{
  PalletError,
};

// use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits:: {
    Zero,
	},
};

mod types;
pub use types::*;
mod constants;
pub use constants::*;
pub mod traits;
pub mod types_nfa;
mod characteristics;
pub use characteristics::*;
mod errors;
pub use errors::*;
pub mod misc;

use types_nfa::{AssetDetails, ClassDetails};

#[cfg(test)]
mod tests;

/// The Account index (aka nonce) type. This stores the number of previous transactions
/// associated with a sender account.
pub type Index = u32;

pub type DispatchResultAs<T> = sp_std::result::Result<T, sp_runtime::DispatchError>;
/// Type alias for `frame_system`'s account id. \
/// The user account identifier type for the runtime.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// Type alias for `frame_system`'s index. \
/// Account index (aka nonce) type. This stores the number of previous transactions associated with a sender account.
pub type IndexOf<T> = <T as frame_system::Config>::Index;
/// Type alias for Mechanic Id with config type
pub type MechanicIdOf<T> = MechanicId<AccountIdOf<T>, IndexOf<T>>;
/// Type alias for ClassDetails with config type
pub type ClassDetailsOf<T> = ClassDetails<AccountIdOf<T>>;
/// Type alias for AssetDetails with config type
pub type AssetDetailsOf<T> = AssetDetails<AccountIdOf<T>, IndexOf<T>>;
/// Type alias for LockResult with config type
pub type LockResultOf<T> = LockResult<AccountIdOf<T>, IndexOf<T>>;

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

/// Represent the origin of lock
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Locker<AccountId, Index> {
  /// Not locked
  None,
  /// Locked by mechanic
  Mechanic(MechanicId<AccountId, Index>),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Structure to represent of the Mechanic Id
pub struct  MechanicId<AccountId, Index>
{
  pub account_id: AccountId,
  pub nonce: Index,
}
impl<AccountId, Index> MechanicId<AccountId, Index>
where AccountId: PartialEq
{
  /// Creates mechanic id from an account id
  pub fn from_account_id<T: frame_system::Config> (account_id: &T::AccountId) -> MechanicId<T::AccountId, T::Index>
  {
    let nonce = <frame_system::Pallet<T>>::account_nonce(account_id);
    MechanicId {
      account_id: account_id.clone(),
      nonce,
    }
  }
  /// Ensure that `who` equal `account_id` of mechanic id
  pub fn ensure_owner(&self, who: &AccountId) -> Result<(), &str> {
    if &self.account_id == who {
      Ok(())
    } else {
      Err("Not owner of mechanic")
    }
  }
}

#[derive(RuntimeDebug, PartialEq)]
/// Result of locking assets
pub enum LockResult<AccountId, Index> {
  /// The asset has been locked for the first time
  Locked(AssetDetails<AccountId, Index>),
  /// The asset already has the required status
  Already(AssetDetails<AccountId, Index>),
}

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Represent an FA or NFA asset id of any type
pub enum AssetId {
  Fa(FungibleAssetId),
  Nfa(NonFungibleClassId, NonFungibleAssetId),
}

#[derive(Copy, Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebug)]
/// Represent a locked FA with amount or NFA asset id
pub enum LockedAccet {
  Fa(FungibleAssetId, FungibleAssetBalance),
  Nfa(NonFungibleClassId, NonFungibleAssetId),
}
