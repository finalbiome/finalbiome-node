use super::*;

pub mod bettor;
pub mod purchased;

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Characteristic {
  Bettor,
  Purchased,
}
