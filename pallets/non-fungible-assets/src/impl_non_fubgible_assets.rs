use super::*;

impl<T: Config> support::NonFungibleAssets for Pallet<T> {
  type ClassId = NonFungibleAssetId;
  type AssetId = NonFungibleClassId;
}
