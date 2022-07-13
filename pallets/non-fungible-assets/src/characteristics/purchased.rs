//! The Purchased Characteristic code
use super::super::*;
use super::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Parameters of the Purchased Characteristic
pub struct Purchased<Key, StringLimit> {
  pub offers: BoundedVec<Offer<Key, StringLimit>, ConstU32<{ u8::MAX as u32 }>>,
}

impl<Key, AttrStringType> AssetCharacteristic for Purchased<Key, AttrStringType> {
    fn is_valid(&self) -> bool {
        // number of offers must be more than 0
        if self.offers.len() == 0 {
          return false
        }
        // price must be more than 0
        if self.offers.iter().any(|o| o.price.is_zero()) {
          return false
        }
        // TODO: check for the existence of an FA
        true
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// An offer of the Purchased Characteristic
pub struct Offer<Key, AttrStringType> {
  pub fa: FungibleAssetId,
  pub price: FungibleAssetBalance,
  pub attributes: BoundedVec<Attribute<Key, AttrStringType>, ConstU32<{ u8::MAX as u32 }>>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// Represent a single attribute as key and value
pub struct Attribute<Key, AttrStringType> {
  pub key: Key,
  pub value: AttributeDetails<AttrStringType>,
}
