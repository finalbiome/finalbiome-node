//! The Purchased Characteristic code
use super::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Parameters of the Purchased Characteristic
pub struct Purchased {
  pub offers: BoundedVec<Offer, ConstU32<{ u8::MAX as u32 }>>,
}

impl<T: Config> AssetCharacteristic<T> for Purchased {
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
        // TODO: check that no attributes with default values (like in class)
        true
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// An offer of the Purchased Characteristic
pub struct Offer {
  pub fa: FungibleAssetId,
  pub price: FungibleAssetBalance,
  pub attributes: AttributeList,
}
