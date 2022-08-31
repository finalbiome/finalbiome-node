use crate::{bettor::Bettor, purchased::Purchased};

use super::*;

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
// #[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ClassDetails<AccountId> {
  pub owner: AccountId,
  /// The total number of outstanding instances of this asset class
  pub instances: u32,
  /// The total number of attributes for this asset class.
  pub attributes: u32,
  /// Name of the Asset. Limited in length by `ClassNameLimit`
  pub name: BoundedVec<u8, DefaultStringLimit>,
  /// Characteristic of bets
  pub bettor: Option<Bettor>,
  /// Characteristic of purchases
  pub purchased: Option<Purchased>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct AssetDetails<AccountId, Index> {
  /// The owner of this asset.
  pub owner: AccountId,
  // Who locked this instance
  pub locked: Locker<AccountId, Index>,
}
