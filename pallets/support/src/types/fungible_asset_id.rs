use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
pub use num_traits::{CheckedAdd, One};
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
	Ord,
	PartialOrd,
	MaxEncodedLen,
	CompactAs,
	Default,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FungibleAssetId(u32);

impl From<u32> for FungibleAssetId {
	#[inline]
	fn from(value: u32) -> Self {
		FungibleAssetId(value)
	}
}

impl core::ops::Deref for FungibleAssetId {
	type Target = u32;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Iterator for FungibleAssetId {
  type Item = Self;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.0.checked_add(1).map(FungibleAssetId)
  }
}

#[cfg(test)]
mod tests {
	use crate::FungibleAssetId;

	#[test]
	fn fa_deref() {
		let a = FungibleAssetId::from(2);
		assert_eq!(2, *a)
	}

	#[test]
	fn fa_next() {
		let mut a = FungibleAssetId::from(2);
		assert_eq!(a.next(), Some(FungibleAssetId::from(3)))
	}
}
