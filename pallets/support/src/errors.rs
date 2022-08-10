use super::*;

#[derive(Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, PalletError)]
pub enum CommonError {
	/// Characteristic is invalid
	WrongCharacteristic,
  /// The bettor characteristic is wrong.
  WrongBettor,
  /// The purchased characteristic is wrong.
  WrongPurchased,
}
