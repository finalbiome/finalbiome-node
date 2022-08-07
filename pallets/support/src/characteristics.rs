use crate::{
  errors::CommonError,
  types::{
    CharacteristicBettor,
    CharacteristicPurchased,
  }
};

use super::*;

pub mod bettor;
pub mod purchased;

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
  /// Check the characteristic and return WrongCharacteristic error
  fn ensure(&self) -> Result<(), CommonError> {
    if !self.is_valid() {
      return Err(CommonError::WrongCharacteristic)
    }
    Ok(())
  }
}

/// Represent a some single characteristic of NFA class
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Characteristic {
  Bettor(CharacteristicBettor),
  Purchased(CharacteristicPurchased),
}
