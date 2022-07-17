use super::*;
use characteristics::*;
use characteristics::bettor::*;
use characteristics::purchased::*;

/// Type of the non-fungible asset instance ids
pub type NonFungibleAssetId = pallet_support::NonFungibleAssetId;
/// Type of the non-fungible class of assets ids
pub type NonFungibleClassId = pallet_support::NonFungibleClassId;
/// Type of the fungible asset id
pub type FungibleAssetId = pallet_support::FungibleAssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance = pallet_support::FungibleAssetBalance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ClassDetails<AccountId, BoundedString> {
  pub owner: AccountId,
  /// The total number of outstanding instances of this asset class
	pub instances: u32,
  /// The total number of attributes for this asset class.
	pub attributes: u32,
  /// Name of the Asset. Limited in length by `ClassNameLimit`
	pub name: BoundedString,
  /// Characteristic of bets
  pub bettor: Option<Bettor>,
  /// Characteristic of purchases
  pub purchased: Option<Purchased>,
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
  bettor: CharacteristicBettor,
  purchased: CharacteristicPurchased,
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
      purchased: None,
    })
  }

  /// Set the Bettor chracteristic of the NFA
  pub fn bettor(mut self, bettor: CharacteristicBettor) -> ClassDetailsBuilderResult<T> {
    if let Some(ref inc_bettor) = bettor {
      if !inc_bettor.is_valid() {
        return Err(Error::<T>::WrongBettor.into())
      }
    }
    self.bettor = bettor;
    Ok(self)
  }

  pub fn purchased(mut self, purchased: CharacteristicPurchased) -> ClassDetailsBuilderResult<T> {
    if let Some(ref inc_purchased) = purchased {
      if !inc_purchased.is_valid() {
        return Err(Error::<T>::WrongPurchased.into())
      }
    }
    self.purchased = purchased;
    Ok(self)
  }

  /// Validation of the all class details.
  fn validate(&self) -> DispatchResult {
    Ok(())
  }

  pub fn build(self) -> DispatchResultAs<ClassDetailsOf<T>> {
    self.validate()?;
    Ok(ClassDetails {
      owner: self.owner,
      name: self.name,
      instances: Zero::zero(),
      attributes: Zero::zero(),
      bettor: None,
      purchased: None,
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
  pub fn build(self) -> DispatchResultAs<AssetDetails<T::AccountId>> {
    Ok(AssetDetails {
      owner: self.owner,
    })
  }
}

// endregion: Builders

pub type ClassNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::ClassNameLimit>;
type ClassDetailsBuilderResult<T> = DispatchResultAs<ClassDetailsBuilder<T>>;
type AssetDetailsBuilderResult<T> = DispatchResultAs<AssetDetailsBuilder<T>>;
pub type ClassDetailsOf<T> = ClassDetails<AccountIdOf<T>, ClassNameLimit<T>>;

pub type CharacteristicBettor = Option<Bettor>;
pub type CharacteristicPurchased = Option<Purchased>;

// region: Genesis Types
pub type GenesisClassesConfigOf<T> = Vec<(NonFungibleClassId, AccountIdOf<T>, Vec<u8>)>;
// endregion: Genesis Types
