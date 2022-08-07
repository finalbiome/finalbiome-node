use pallet_support::{AssetId, DefaultListLengthLimit, BETTOR_MAX_NUMBER_OF_ROUNDS};
use scale_info::TypeInfo;
use codec::{Decode, Encode, MaxEncodedLen};

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
/// Describes types of mechanics
pub enum Mechanic {
    /// NFA purchase
    BuyNfa,
    /// NFA bet
    Bet,
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(test, derive(Debug, PartialEq))]
/// The details of the active mechanic
pub(crate) struct MechanicDetails<AccountId, BlockNumber> {
	/// Owner of the mechanic
	pub owner: AccountId,
	/// Mechain timeout id
	pub timeout_id: Option<BlockNumber>,
	/// List of assets locked by mechanic
	pub locked: BoundedVec<AssetId, DefaultListLengthLimit>,
	// store a type of mechanic with data
	pub data: MechanicData,
}
impl<AccountId, BlockNumber> MechanicDetails<AccountId, BlockNumber> {
	pub fn new(owner: AccountId) -> Self {
		Self { 
			owner,
			timeout_id: Default::default(),
			locked: Default::default(),
			data: Default::default(),
		 }
	}
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum MechanicData {
	/// None of mechanic data
	None,
	/// Data of the BuyNfa mechanic. Stub - no data needed
	BuyNfa,
	/// Data of the Bet mechanic
	Bet(MechanicDataBet)
}
impl Default for MechanicData {
	fn default() -> Self {
		MechanicData::None
	}
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Data for the Bet machanic hold results of the outcomes of rounds played.
pub(crate) struct MechanicDataBet {
	/// Each index of `outcomes` represent the played round and a value - index of the dropped variant in the bettor respectively
	pub outcomes: MechanicDataBetOutcomes,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Results of the bettor round
pub(crate) enum BetResult {
  Won,
  Lost,
  Draw,
}

pub(crate) type MechanicDetailsOf<T> = MechanicDetails<AccountIdOf<T>, BlockNumberFor<T>>;
/// Each index of `outcomes` represent the played round and a value - index of the dropped variant in the bettor respectively
pub(crate) type MechanicDataBetOutcomes = BoundedVec<u32, ConstU32<BETTOR_MAX_NUMBER_OF_ROUNDS>>;
