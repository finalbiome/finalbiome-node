use super::*;

// use frame_system::Account;

// /// Identifier of the asset.
// pub(super) type AssetId = impl Member
//   + Parameter
//   + Default
//   + Copy
//   + HasCompact
//   + MaybeSerializeDeserialize
//   + MaxEncodedLen
//   + TypeInfo;

// type BalanceOf<F, T> = <F as fungible::Inspect<AccountIdOf<T>>>::Balance;
// pub type OrganizationIdOf<T> = <T as pallet::Config>::Balance;

#[must_use]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// Consequence of a decrease in the amount of assets
pub enum TopUpConsequence {
  /// The amount to top up which need add in the next block but not final (does not reach the cup)
  TopUp(FungibleAssetBalance),
  /// The amount to top up which need add in the next block and reach the cup
  TopUpFinal(FungibleAssetBalance),
  None,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetAccount {
	/// The balance
	pub(super) balance: FungibleAssetBalance,
  /// The reason for the existence of the account.
	pub(super) reason: ExistenceReason,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ExistenceReason {
	#[codec(index = 0)]
	Consumer,
	#[codec(index = 1)]
	Sufficient,
	#[codec(index = 2)]
	DepositHeld(FungibleAssetBalance),
	#[codec(index = 3)]
	DepositRefunded,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<AccountId, BoundedString> {
  pub(super) owner: AccountId,
  /// The total supply across all accounts.
	pub(super) supply: FungibleAssetBalance,
  /// The total number of accounts.
	pub(super) accounts: u32,
  /// The total number of references.  \
  /// When an asset used in NFA or some mechanics it can't be destroyed
	pub(super) references: u32,
  /// Name of the Asset. Limited in length by `NameLimit`.
	pub(super) name: BoundedString,
  /// Characteristic of auto generation
  pub(super) top_upped: Option<TopUppedFA>,
  /// Characteristic of global limit of the FA
  pub(super) cup_global: Option<CupFA>,
  /// Characteristic of an account limit of the FA
  pub(super) cup_local: Option<CupFA>,
}

impl<AccountId, BoundedString> AssetDetails<AccountId, BoundedString> {
    /// Returns the amount to top up in the next block \
    /// If None - no top up needed \
    /// `current_balance` - current balance of given account
    pub fn next_step_topup(&self, current_balance: FungibleAssetBalance) -> TopUpConsequence {
      use TopUpConsequence::*;
      if let Some(topup) = &self.top_upped {
        if topup.speed > Zero::zero() {
          if let Some(cup) = &self.cup_local {
            let diff = cup.amount.saturating_sub(&current_balance);
            if diff == Zero::zero() {
              return None
            } else if diff > topup.speed {
              return TopUp(topup.speed)
            } else {
              return TopUpFinal(diff)
            }
          }
        }
      }
      None
    }
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct TopUppedFA {
  /// Speed of top upped (recovery speed) as `N` per block
  pub speed: FungibleAssetBalance,
}

impl AssetCharacteristic for TopUppedFA {
  fn is_valid(&self) -> bool {
      self.speed > Zero::zero()
  }
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct CupFA {
  /// The limit of the FA
  pub amount: FungibleAssetBalance
}
impl AssetCharacteristic for CupFA {
    fn is_valid(&self) -> bool {
        self.amount > Zero::zero()
    }
}

pub trait AssetCharacteristic {
  fn is_valid(&self) -> bool;
}

#[derive(Default)]
pub(super) struct AssetDetailsBuilder<T: Config> {
    owner: T::AccountId,
    name: NameLimit<T>,
    top_upped: Option<TopUppedFA>,
    cup_global: Option<CupFA>,
    cup_local: Option<CupFA>,
}

impl<T: pallet::Config> AssetDetailsBuilder<T> {
  pub fn new(owner: T::AccountId, name: Vec<u8>) -> AssetDetailsBuilderResult<T> {
    let bounded_name = name.try_into();
			let bounded_name = match bounded_name {
				Ok(name) => name,
				Err(_) => return Err(Error::<T>::AssetNameTooLong.into()),
			};
    Ok(AssetDetailsBuilder {
      owner,
      name: bounded_name,
      top_upped: None,
      cup_global: None,
      cup_local: None,
    })
  }

  /// Set the top upped characteristic of the FA
  pub fn top_upped(mut self, top_upped: Option<TopUppedFA>) -> AssetDetailsBuilderResult<T> {
    if top_upped.is_some() && !top_upped.as_ref().unwrap().is_valid() {
      return Err(Error::<T>::ZeroTopUpped.into())
    }
    self.top_upped = top_upped;
    Ok(self)
  }

  /// Set the global cup characteristic
  pub fn cup_global(mut self, cup: Option<CupFA>) -> AssetDetailsBuilderResult<T> {
    if cup.is_some() && !cup.as_ref().unwrap().is_valid() {
      return Err(Error::<T>::ZeroGlobalCup.into())
    }
    self.cup_global = cup;
    Ok(self)
  }

  /// Set the local cup characteristic
  pub fn cup_local(mut self, cup: Option<CupFA>) -> AssetDetailsBuilderResult<T> {
    if cup.is_some() && !cup.as_ref().unwrap().is_valid() {
      return Err(Error::<T>::ZeroLocalCup.into())
    }
    self.cup_local = cup;
    Ok(self)
  }

  /// Validation of the all asset details.
  /// Rise the panic if something wrong
  pub fn validate(&self) -> DispatchResult {
    if self.top_upped.is_some() && self.cup_local.is_none() {
        return Err(Error::<T>::TopUppedWithNoCup.into())
      }
    Ok(())
  }

  pub fn build(self) -> DispatchResultAs<AssetDetails<T::AccountId, NameLimit<T>>> {
    self.validate()?;
    Ok(AssetDetails {
      owner: self.owner,
      supply: Zero::zero(),
      accounts: Zero::zero(),
      references: Zero::zero(),
      name: self.name,
      top_upped: self.top_upped,
      cup_global: self.cup_global,
      cup_local: self.cup_local,
    })
  }
  
}

/// Type of the fungible asset's ids
pub type AssetId = pallet_support::FungibleAssetId;
pub type AssetBalance = pallet_support::FungibleAssetBalance;

pub type NameLimit<T> = BoundedVec<u8, <T as pallet::Config>::NameLimit>;

type AssetDetailsBuilderResult<T> = DispatchResultAs<AssetDetailsBuilder<T>>;

pub type GenesisAssetsConfigOf<T> = Vec<(AssetId, AccountIdOf<T>, Vec<u8>, Option<FungibleAssetBalance>, Option<FungibleAssetBalance>, Option<FungibleAssetBalance>)>;
