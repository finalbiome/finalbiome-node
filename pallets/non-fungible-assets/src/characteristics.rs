pub mod bettor;

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
}

