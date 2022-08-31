use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Type of the fungible class id
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
pub struct NonFungibleClassId(u32);

impl From<u32> for NonFungibleClassId {
  #[inline]
  fn from(value: u32) -> Self {
    NonFungibleClassId(value)
  }
}

impl core::ops::Deref for NonFungibleClassId {
  type Target = u32;
  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Iterator for NonFungibleClassId {
  type Item = Self;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.0.checked_add(1).map(NonFungibleClassId)
  }
}

#[cfg(test)]
mod tests {
  use super::NonFungibleClassId;

  #[test]
  fn fa_deref() {
    let a = NonFungibleClassId::from(2);
    assert_eq!(2, *a)
  }

  #[test]
  fn fa_next() {
    let mut a = NonFungibleClassId::from(2);
    assert_eq!(a.next(), Some(NonFungibleClassId::from(3)))
  }
}
