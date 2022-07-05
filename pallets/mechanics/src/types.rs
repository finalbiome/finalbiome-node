use super::*;

/// Type of the fungible asset id
pub type FungibleAssetId<T> = <<T as pallet::Config>::FungibleAssets as support::FungibleAssets>::AssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance<T> = <<T as pallet::Config>::FungibleAssets as support::FungibleAssets>::Balance;
/// Type of the non-fungible asset id
pub type NonFungibleClassId<T> = <<T as pallet::Config>::NonFungibleAssets as support::NonFungibleAssets>::ClassId;
/// The units in which we record balances of the fungible assets
pub type NonFungibleAssetId<T> = <<T as pallet::Config>::NonFungibleAssets as support::NonFungibleAssets>::AssetId;
