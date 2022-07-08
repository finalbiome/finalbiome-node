use super::*;

use characteristics::*;
use characteristics::bettor::*;

/// Type of the non-fungible asset instance ids
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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetDetails<AccountId> {
  /// The owner of this asset.
  pub(super) owner: AccountId,
}

// region: Builders
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

pub struct AssetDetailsBuilder<T: Config> {
  owner: T::AccountId,
}
impl<T: pallet::Config> AssetDetailsBuilder<T> {
  pub fn new(owner: T::AccountId) -> AssetDetailsBuilderResult<T> {
    Ok(AssetDetailsBuilder {
      owner
    })
  }
  pub fn build(self) -> Result<AssetDetails<T::AccountId>, DispatchError> {
    Ok(AssetDetails {
      owner: self.owner,
    })
  }
}

// endregion: Builders

pub type ClassNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::ClassNameLimit>;
type ClassDetailsBuilderResult<T> = Result<ClassDetailsBuilder<T>, DispatchError>;
type AssetDetailsBuilderResult<T> = Result<AssetDetailsBuilder<T>, DispatchError>;
pub type BettorOutcomeNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::BettorOutcomeNameLimit>;
pub type BettorOutcomeName<T> = BoundedVec<u8,<T as pallet::Config>::BettorOutcomeNameLimit>;
