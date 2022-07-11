pub mod bettor;
pub mod purchased;

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
}

