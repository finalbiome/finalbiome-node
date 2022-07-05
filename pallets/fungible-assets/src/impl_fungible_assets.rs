use super::*;

impl<T: Config> support::FungibleAssets for Pallet<T> {
  type AssetId = AssetId;
  type Balance = T::Balance;
}
