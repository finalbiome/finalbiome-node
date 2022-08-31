use codec::{CompactAs, Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
pub use num_traits::{CheckedAdd, CheckedSub, SaturatingAdd, SaturatingSub};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Zero;

/// The units in which we record balances of the fungible assets
#[derive(
  Copy,
  Clone,
  Encode,
  Decode,
  TypeInfo,
  MaxEncodedLen,
  Eq,
  PartialEq,
  RuntimeDebug,
  PartialOrd,
  Ord,
  CompactAs,
  Default,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FungibleAssetBalance(u128);

impl From<u128> for FungibleAssetBalance {
  #[inline]
  fn from(value: u128) -> Self {
    FungibleAssetBalance(value)
  }
}

impl core::ops::Deref for FungibleAssetBalance {
  type Target = u128;
  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Zero for FungibleAssetBalance {
  #[inline]
  fn zero() -> Self {
    FungibleAssetBalance(0)
  }
  #[inline]
  fn is_zero(&self) -> bool {
    self.0 == 0
  }
}
impl sp_std::ops::Add for FungibleAssetBalance {
  type Output = Self;
  #[inline]
  fn add(self, rhs: Self) -> Self::Output {
    FungibleAssetBalance(self.0.add(rhs.0))
  }
}

impl sp_std::ops::Sub for FungibleAssetBalance {
  type Output = Self;
  #[inline]
  fn sub(self, rhs: Self) -> Self::Output {
    FungibleAssetBalance(self.0.sub(rhs.0))
  }
}

impl SaturatingAdd for FungibleAssetBalance {
  #[inline]
  fn saturating_add(&self, v: &Self) -> Self {
    FungibleAssetBalance(self.0.saturating_add(v.0))
  }
}
impl SaturatingSub for FungibleAssetBalance {
  #[inline]
  fn saturating_sub(&self, v: &Self) -> Self {
    FungibleAssetBalance(self.0.saturating_sub(v.0))
  }
}

impl CheckedAdd for FungibleAssetBalance {
  #[inline]
  fn checked_add(&self, v: &Self) -> Option<Self> {
    self.0.checked_add(v.0).map(FungibleAssetBalance)
  }
}

impl CheckedSub for FungibleAssetBalance {
  #[inline]
  fn checked_sub(&self, v: &Self) -> Option<Self> {
    self.0.checked_sub(v.0).map(FungibleAssetBalance)
  }
}

#[cfg(test)]
mod tests {
  use crate::FungibleAssetBalance;

  #[test]
  fn fab_deref() {
    let a = FungibleAssetBalance::from(2);
    assert_eq!(2, *a)
  }
  #[test]
  fn fab_add() {
    let a = FungibleAssetBalance::from(1);
    let b = FungibleAssetBalance::from(2);
    let c = FungibleAssetBalance::from(3);
    assert_eq!(a + b, c)
  }
  #[test]
  fn fab_sub() {
    let a = FungibleAssetBalance::from(1);
    let b = FungibleAssetBalance::from(2);
    let c = FungibleAssetBalance::from(3);
    assert_eq!(c - b, a)
  }
}
