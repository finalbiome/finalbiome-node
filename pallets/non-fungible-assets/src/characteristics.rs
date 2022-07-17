use super::*;

pub mod bettor;
pub mod purchased;

pub trait AssetCharacteristic<T: Config> {
  fn is_valid(&self) -> bool;
  /// Check the characteristic and return WrongCharacteristic error
  fn ensure(&self) -> DispatchResult {
    if !self.is_valid() {
      return Err(Error::<T>::WrongCharacteristic.into())
    }
    Ok(())
  }
}

/// Represent a some single characteristic of NFA class
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Characteristic {
  Bettor(CharacteristicBettor),
  Purchased(CharacteristicPurchased),
}
