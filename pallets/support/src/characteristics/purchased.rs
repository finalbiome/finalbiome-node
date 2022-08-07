//! The Purchased Characteristic code
use super::*;

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebug)]
/// Parameters of the Purchased Characteristic
pub struct Purchased {
  pub offers: BoundedVec<Offer, DefaultListLengthLimit>,
}

impl AssetCharacteristic for Purchased {
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
    fn ensure(&self) -> Result<(), CommonError> {
      if !self.is_valid() {
        return Err(CommonError::WrongPurchased)
      }
      Ok(())
    }
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebug)]
/// An offer of the Purchased Characteristic
pub struct Offer {
  pub fa: FungibleAssetId,
  pub price: FungibleAssetBalance,
  pub attributes: AttributeList,
}
