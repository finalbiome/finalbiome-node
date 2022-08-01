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
pub enum MechanicType {
    /// Creating an NFA
    CreateNFA,
    /// Destroying an NFA
    DestroyNFA,
}
