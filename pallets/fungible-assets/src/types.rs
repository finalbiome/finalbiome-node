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

pub(super) type AssetAccountOf<T> = AssetAccount<<T as Config>::Balance>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetAccount<Balance> {
	/// The balance
	pub(super) balance: Balance,
  /// The reason for the existence of the account.
	pub(super) reason: ExistenceReason<Balance>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ExistenceReason<Balance> {
	#[codec(index = 0)]
	Consumer,
	#[codec(index = 1)]
	Sufficient,
	#[codec(index = 2)]
	DepositHeld(Balance),
	#[codec(index = 3)]
	DepositRefunded,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<AccountId, Balance, BoundedString> {
  pub(super) owner: AccountId,
  /// The total supply across all accounts.
	pub(super) supply: Balance,
  /// The total number of accounts.
	pub(super) accounts: u32,
  /// Name of the Asset. Limited in length by `NameLimit`.
	pub(super) name: BoundedString,
  /// Characteristic of auto generation
  pub(super) top_upped: Option<TopUppedFA>,
  /// Characteristic of global limit of the FA
  pub(super) cup_global: Option<CupFA>,
  /// Characteristic of an account limit of the FA
  pub(super) cup_local: Option<CupFA>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct TopUppedFA {
  /// Speed of top upped (recovery speed) as `N` per block
  pub speed: u32,
}

impl AssetCharacteristic for TopUppedFA {
  fn is_valid(&self) -> bool {
      self.speed > 0
  }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct CupFA {
  /// The limit of the FA
  pub amount: u64
}
impl AssetCharacteristic for CupFA {
    fn is_valid(&self) -> bool {
        self.amount > 0
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
  pub fn validate(&self) -> Result<(), sp_runtime::DispatchError> {
    if self.top_upped.is_some() && self.cup_local.is_none() {
        return Err(Error::<T>::TopUppedWithNoCup.into())
      }
    Ok(())
  }

  pub fn build(self) -> Result<AssetDetails<T::AccountId, T::Balance, NameLimit<T>>, sp_runtime::DispatchError> {
    self.validate()?;
    Ok(AssetDetails {
      owner: self.owner,
      supply: Zero::zero(),
      accounts: Zero::zero(),
      name: self.name,
      top_upped: self.top_upped,
      cup_global: self.cup_global,
      cup_local: self.cup_local,
    })
  }
  
}

/// Type of the fungible asset's ids
pub type AssetId = u32;

pub type NameLimit<T> = BoundedVec<u8, <T as pallet::Config>::NameLimit>;

type AssetDetailsBuilderResult<T> = Result<AssetDetailsBuilder<T>, sp_runtime::DispatchError>;

// pub(super) type DepositBalanceOf<T = ()> =
// 	<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;
