use codec::{Decode, Encode, MaxEncodedLen};
use pallet_support::{
  DefaultListLengthLimit, GamerAccount, IndexOf, LockedAccet, BETTOR_MAX_NUMBER_OF_ROUNDS,
};
use scale_info::TypeInfo;

use super::*;

/// Type of the non-fungible asset instance ids
pub type NonFungibleAssetId = pallet_support::NonFungibleAssetId;
/// Type of the non-fungible class of assets ids
pub type NonFungibleClassId = pallet_support::NonFungibleClassId;
/// Type of the fungible asset id
pub type FungibleAssetId = pallet_support::FungibleAssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance = pallet_support::FungibleAssetBalance;

/// Bounded vector of NFA ids
pub type NonFungibleAssetIds<T> = BoundedVec<NonFungibleAssetId, <T as Config>::AssetsListLimit>;

#[derive(PartialEq, Eq)]
/// Describes types of mechanics
pub enum Mechanic {
  /// NFA purchase
  BuyNfa,
  /// NFA bet
  Bet,
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug)]
/// The details of the active mechanic
pub struct MechanicDetails<AccountId, BlockNumber> {
  /// Owner of the mechanic
  pub owner: GamerAccount<AccountId>,
  /// Mechain timeout id
  pub timeout_id: BlockNumber,
  /// List of assets locked by mechanic
  pub locked: BoundedVec<LockedAccet, DefaultListLengthLimit>,
  // store a type of mechanic with data
  pub data: MechanicData,
}
impl<AccountId, BlockNumber> MechanicDetails<AccountId, BlockNumber>
where
  AccountId: Clone,
  BlockNumber: Copy,
{
  /// Returns a storage key for the [Pallet::Timeouts] storage
  pub fn get_tiomeout_strorage_key(
    &self,
    nonce: Index,
  ) -> (BlockNumber, GamerAccount<AccountId>, Index) {
    (self.timeout_id, self.owner.clone(), nonce)
  }
}

/// Build a Mechanic Details data.  \
/// Used for construction new mechanic data struct.
pub(crate) struct MechanicDetailsBuilder {}
impl MechanicDetailsBuilder {
  pub fn build<T: pallet::Config>(
    owner: GamerAccount<T::AccountId>,
    data: MechanicData,
  ) -> MechanicDetailsOf<T> {
    let timeout_id = Pallet::<T>::calc_timeout_block();
    MechanicDetails {
      owner,
      timeout_id,
      locked: Default::default(),
      data,
    }
  }
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, PartialEq, Eq)]
pub enum MechanicData {
  /// Data of the BuyNfa mechanic. Stub - no data needed
  BuyNfa,
  /// Data of the Bet mechanic
  Bet(MechanicDataBet),
}
impl From<&MechanicData> for Mechanic {
  fn from(value: &MechanicData) -> Self {
    match value {
      MechanicData::Bet(_) => Mechanic::Bet,
      MechanicData::BuyNfa => Mechanic::BuyNfa,
    }
  }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
/// Data for the Bet machanic hold results of the outcomes of rounds played.
pub struct MechanicDataBet {
  /// Each index of `outcomes` represent the played round and a value - index of the dropped
  /// variant in the bettor respectively
  pub outcomes: MechanicDataBetOutcomes,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Results of the bettor round
pub enum BetResult {
  Won,
  Lost,
  Draw,
}

#[derive(RuntimeDebug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
/// The data that is transmitted to upgrade the mechanic.
pub struct MechanicUpgradeData<AccountId, Index> {
  pub mechanic_id: MechanicId<AccountId, Index>,
  /// Payload of the data.  \
  /// Depends on the type of mechanics
  pub payload: MechanicUpgradePayload,
}

#[derive(RuntimeDebug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
/// Payload of the data for upgrade specific mechanic
pub enum MechanicUpgradePayload {
  /// For the Bet mechanic no need any data
  Bet,
}
impl From<&MechanicUpgradePayload> for Mechanic {
  fn from(data: &MechanicUpgradePayload) -> Self {
    match data {
      MechanicUpgradePayload::Bet => Mechanic::Bet,
    }
  }
}

#[derive(Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum EventMechanicStopReason<AccountId, BlockNumber> {
  /// Needs a mechanics upgrade
  UpgradeNeeded(MechanicDetails<AccountId, BlockNumber>),
}

#[derive(Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum EventMechanicResultData {
  /// Hold a minted asset id
  BuyNfa(NonFungibleAssetId),
  /// Hold a final outcoms of Bet mechanic
  Bet(EventMechanicResultDataBet),
}

#[derive(Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct EventMechanicResultDataBet {
  /// Hold a final outcomes of Bet mechanic
  pub outcomes: MechanicDataBetOutcomes,
  /// Hold a final result of bettor mechanic
  pub result: BetResult,
}

/// Actions that are performed with the assets at the time of the destruction of the mechanics
pub(crate) enum AssetAction {
  /// All assets will be released and available to the user
  Release,
  /// All assets will be burned
  Burn,
}

pub(crate) type MechanicDetailsOf<T> = MechanicDetails<AccountIdOf<T>, BlockNumberFor<T>>;
/// Each index of `outcomes` represent the played round and a value - index of the dropped variant
/// in the bettor respectively
pub(crate) type MechanicDataBetOutcomes = BoundedVec<u32, ConstU32<BETTOR_MAX_NUMBER_OF_ROUNDS>>;

pub(crate) type MechanicUpgradeDataOf<T> = MechanicUpgradeData<AccountIdOf<T>, IndexOf<T>>;
pub(crate) type EventMechanicStopReasonOf<T> =
  EventMechanicStopReason<AccountIdOf<T>, BlockNumberFor<T>>;

/// Type of data that hold a result of finished mechanic.
///
/// This is necessary because the mechanic resets all internal data when it finished.
pub(crate) type EventMechanicResult = Option<EventMechanicResultData>;
