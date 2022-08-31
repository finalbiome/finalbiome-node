//! The Bettor Characteristics code
use super::*;

/// Parameters of the Bettor Characteristic
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, RuntimeDebug)]
pub struct Bettor {
  /// Options of outcomes
  pub outcomes: Outcomes,
  pub winnings: BoundedVec<BettorWinning, DefaultListLengthLimit>,
  pub rounds: RoundsCount,
  pub draw_outcome: DrawOutcomeResult,
}

impl AssetCharacteristic for Bettor {
  fn is_valid(&self) -> bool {
    // count of outcomes must be more than 1 (else it is not controlled mint)
    if self.outcomes.len() < 2 {
      return false;
    }
    // outcome probabilities must be more than 0
    if !self.outcomes.iter().all(|p| p.probability != 0) {
      return false;
    }
    // count if winings must be more than 0
    if self.winnings.len() == 0 {
      return false;
    }
    // rounds must be in [1..BETTOR_MAX_NUMBER_OF_ROUNDS]
    if self.rounds < 1 || self.rounds > BETTOR_MAX_NUMBER_OF_ROUNDS {
      return false;
    }
    // outcomes must have at least one win and one lose results
    let mut win_results = 0;
    let mut lose_results = 0;
    for o in self.outcomes.iter() {
      match o.result {
        OutcomeResult::Win => win_results += 1,
        OutcomeResult::Lose => lose_results += 1,
        OutcomeResult::Draw => (),
      }
    }
    if win_results == 0 || lose_results == 0 {
      return false;
    }

    // Assets must exist and FAs should have amount more than 0
    // for winning in self.winnings.iter() {
    // TODO: check asset existence
    // TODO: FAs should have amount more than 0

    // FAs should have amount more than 0
    // if let BettorWinning::FA(id, amount) = winning {
    //   if amount == Zero::zero() {
    //     return false
    //   }2
    // }
    // }
    true
  }
  fn ensure(&self) -> Result<(), CommonError> {
    if !self.is_valid() {
      return Err(CommonError::WrongBettor);
    }
    Ok(())
  }
}

#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub struct BettorOutcome {
  /// Name of the outcome
  pub name: BoundedVec<u8, DefaultStringLimit>,
  /// The probability of the outcome
  pub probability: u32,
  /// Result of the current outcome option
  pub result: OutcomeResult,
}

/// A type of the asset with given params of winning results
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub enum BettorWinning {
  /// Fungible asset \
  /// Represented as (FA id, amount)
  Fa(FungibleAssetId, FungibleAssetBalance),
  /// Non-fungible asset \
  /// Represented as (NFA id)
  Nfa(NonFungibleClassId),
}

#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub enum DrawOutcomeResult {
  Win,
  Lose,
  Keep,
}

#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub enum OutcomeResult {
  Win,
  Lose,
  Draw,
}

/// Type of the rounds count
pub type RoundsCount = u32;

pub type Outcomes = BoundedVec<BettorOutcome, DefaultListLengthLimit>;
