//! The Bettor Characteristics code
use super::super::*;
use super::*;

/// Parameters of the Bettor Characteristic
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Bettor<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance, BoundedName> {
  pub outcomes: WeakBoundedVec<BettorOutcome<BoundedName>, ConstU32<{ u8::MAX as u32 }>>,
  pub winnings: WeakBoundedVec<BettorWinning<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance>, ConstU32<{ u8::MAX as u32 }>>,
  pub rounds: u8,
  pub draw_outcome: DrawOutcomeResult,
}

impl<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance, BoundedName> AssetCharacteristic for Bettor<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance, BoundedName> {
  fn is_valid(&self) -> bool {
      // count of outcomes must be more than 0
      if self.outcomes.len() == 0 {
        return false
      }
      // outcome probabilities must be in range 0..100
      if !self.outcomes.iter().all(|p| p.probability <=100) {
        return false
      }
      // sum of all probabilities should be euqal 100
      if self.outcomes.iter().map(|p| p.probability as u32).sum::<u32>() != 100u32 {
        return false
      }
      // count if winings must be more than 0
      if self.winnings.len() == 0 {
        return false
      }
      // Assets must exist and FAs should have amount more than 0
      // for winning in self.winnings.iter() {
        // TODO: check asset existence
        // TODO: FAs should have amount more than 0

        // FAs should have amount more than 0
        // if let BettorWinning::FA(id, amount) = winning {
        //   if amount == Zero::zero() {
        //     return false
        //   }
        // }
      // }
      true
  }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct BettorOutcome<BoundedName> {
  /// Name of the outcome
  pub name: BoundedName,
  /// The probability of the outcome
  pub probability: u8,
}

/// A type of the asset with given params of winning results
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum BettorWinning<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance> {
  /// Fungible asset \
  /// Represented as (FA id, amount)
  FA(FungibleAssetId, FungibleAssetBalance),
  /// Non-fungible asset \
  /// Represented as (NFA id)
  NFA(NonFungibleClasstId),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum DrawOutcomeResult {
  Win,
  Lose,
  Keep,
}
