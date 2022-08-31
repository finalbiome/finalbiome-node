use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Type of the fungible asset id
#[derive(
  Copy,
  Clone,
  Encode,
  Decode,
  RuntimeDebug,
  TypeInfo,
  Eq,
  PartialEq,
  MaxEncodedLen,
  CompactAs,
  Default,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NonFungibleAssetId(u32);

impl From<u32> for NonFungibleAssetId {
  #[inline]
  fn from(value: u32) -> Self {
    NonFungibleAssetId(value)
  }
}

impl core::ops::Deref for NonFungibleAssetId {
  type Target = u32;
  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Iterator for NonFungibleAssetId {
  type Item = Self;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.0.checked_add(1).map(NonFungibleAssetId)
  }
}

#[cfg(test)]
mod tests {
  use super::NonFungibleAssetId;

  #[test]
  fn fa_deref() {
    let a = NonFungibleAssetId::from(2);
    assert_eq!(2, *a)
  }

  #[test]
  fn fa_next() {
    let mut a = NonFungibleAssetId::from(2);
    assert_eq!(a.next(), Some(NonFungibleAssetId::from(3)))
  }
}
