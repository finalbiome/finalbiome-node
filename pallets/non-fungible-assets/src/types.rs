use super::*;
use pallet_support::{
  Locker,
  CharacteristicBettor, CharacteristicPurchased,
  AssetCharacteristic, DefaultStringLimit,
};

/// Type of the non-fungible asset instance ids
pub type NonFungibleAssetId = pallet_support::NonFungibleAssetId;
/// Type of the non-fungible class of assets ids
pub type NonFungibleClassId = pallet_support::NonFungibleClassId;
/// Type of the fungible asset id
pub type FungibleAssetId = pallet_support::FungibleAssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance = pallet_support::FungibleAssetBalance;

// region: Builders
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ClassDetailsBuilder<T: Config> {
  owner: T::AccountId,
  name: BoundedVec<u8, DefaultStringLimit>,
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
      AssetCharacteristic::ensure(inc_bettor)
        .map_err::<Error<T>, _>(Into::into)
        .map_err::<DispatchError, _>(Into::into)?;
    }
    self.bettor = bettor;
    Ok(self)
  }

  pub fn purchased(mut self, purchased: CharacteristicPurchased) -> ClassDetailsBuilderResult<T> {
    if let Some(ref inc_purchased) = purchased {
      AssetCharacteristic::ensure(inc_purchased)
        .map_err::<Error<T>, _>(Into::into)
        .map_err::<DispatchError, _>(Into::into)?;
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
  pub fn build(self) -> DispatchResultAs<AssetDetails<T::AccountId, T::Index>> {
    Ok(AssetDetails {
      owner: self.owner,
      locked: Locker::None,
    })
  }
}

// endregion: Builders

type ClassDetailsBuilderResult<T> = DispatchResultAs<ClassDetailsBuilder<T>>;
type AssetDetailsBuilderResult<T> = DispatchResultAs<AssetDetailsBuilder<T>>;
pub type ClassDetailsOf<T> = ClassDetails<AccountIdOf<T>>;

// region: Genesis Types
pub type GenesisClassesConfigOf<T> = Vec<(NonFungibleClassId, AccountIdOf<T>, Vec<u8>)>;
pub type GenesisNumberAttributesConfig = Vec<(NonFungibleClassId, Vec<u8>, u32, Option<u32>)>;
pub type GenesisTextAttributesConfig = Vec<(NonFungibleClassId, Vec<u8>, Vec<u8>)>;
/// key, num_value, num_max, text_value
pub type GenesisCommonAttributesList = Vec<(Vec<u8>, Option<u32>, Option<u32>, Option<Vec<u8>>)>;
pub type GenesisPurchasedClassesConfig = Vec<(NonFungibleClassId, FungibleAssetId, FungibleAssetBalance, GenesisCommonAttributesList)>;
// endregion: Genesis Types
