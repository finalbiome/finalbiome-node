use super::*;

/// Type of the non-fungible asset ids
pub type NonFungibleAssetId = u32;

/// Type of the non-fungible class of assets ids
pub type NonFungibleClassId = u32;

/// Type of the fungible asset id
pub type FungibleAssetId<T> = <<T as pallet::Config>::FungibleAssets as support::FungibleAssets>::AssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance<T> = <<T as pallet::Config>::FungibleAssets as support::FungibleAssets>::Balance;


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ClassDetails<AccountId, BoundedString, FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance, BoundedName> {
  pub(super) owner: AccountId,
  /// The total number of outstanding instances of this asset class
	pub(super) instances: u32,
  /// Name of the Asset. Limited in length by `ClassNameLimit`
	pub(super) name: BoundedString,
  /// Characteristic of bets
  pub(super) bettor: Option<Bettor<FungibleAssetId, NonFungibleClasstId, FungibleAssetBalance, BoundedName>>,
}

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

/// A type of the asset with given perams of winning results
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

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
}

#[derive(RuntimeDebug, PartialEq)]
pub struct ClassDetailsBuilder<T: Config> {
  owner: T::AccountId,
  name: ClassNameLimit<T>,
  bettor: Option<Bettor<FungibleAssetId<T>, NonFungibleClassId, FungibleAssetBalance<T>, BettorOutcomeName<T>>>,
}
impl<T: pallet::Config> ClassDetailsBuilder<T> {
  pub fn new(owner: T::AccountId, name: Vec<u8>) -> ClassDetailsBuilderResult<T> {
    let name = name.try_into();
    let name = match name {
      Ok(name) => name,
      Err(_) => return Err(Error::<T>::ClassNameTooLong.into()),
    };
    Ok(ClassDetailsBuilder {
      owner,
      name,
      bettor: None,
    })
  }

  /// Set the Bettor chracteristic of the NFA
  pub fn bettor(mut self, bettor: Option<Bettor<FungibleAssetId<T>, NonFungibleClassId, FungibleAssetBalance<T>, BettorOutcomeName<T>>>) -> ClassDetailsBuilderResult<T> {
    if let Some(ref inc_bettor) = bettor {
      if !inc_bettor.is_valid() {
        return Err(Error::<T>::WrongBettor.into())
      }
    }
    self.bettor = bettor;
    Ok(self)
  }

  /// Validation of the all class details.
  fn validate(&self) -> DispatchResult {
    Ok(())
  }

  pub fn build(self) -> Result<ClassDetails<T::AccountId, ClassNameLimit<T>, FungibleAssetId<T>, NonFungibleClassId, FungibleAssetBalance<T>, BettorOutcomeName<T>>, DispatchError> {
    self.validate()?;
    Ok(ClassDetails {
      owner: self.owner,
      name: self.name,
      instances: Zero::zero(),
      bettor: None,
    })
  }
}

pub type ClassNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::ClassNameLimit>;
type ClassDetailsBuilderResult<T> = Result<ClassDetailsBuilder<T>, DispatchError>;
pub type BettorOutcomeNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::BettorOutcomeNameLimit>;
pub type BettorOutcomeName<T> = BoundedVec<u8,<T as pallet::Config>::BettorOutcomeNameLimit>;
