//! The Bettor Characteristics code
use super::*;
use pallet_support::DefaultStringLimit;

/// Parameters of the Bettor Characteristic
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Bettor {
  pub outcomes: BoundedVec<BettorOutcome, ConstU32<{ u8::MAX as u32 }>>,
  pub winnings: BoundedVec<BettorWinning, ConstU32<{ u8::MAX as u32 }>>,
  pub rounds: u8,
  pub draw_outcome: DrawOutcomeResult,
}

impl AssetCharacteristic for Bettor {
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
pub struct BettorOutcome {
  /// Name of the outcome
  pub name: BoundedVec<u8, DefaultStringLimit>,
  /// The probability of the outcome
  pub probability: u8,
}

/// A type of the asset with given params of winning results
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum BettorWinning {
  /// Fungible asset \
  /// Represented as (FA id, amount)
  Fa(FungibleAssetId, FungibleAssetBalance),
  /// Non-fungible asset \
  /// Represented as (NFA id)
  Nfa(NonFungibleClassId),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum DrawOutcomeResult {
  Win,
  Lose,
  Keep,
}
