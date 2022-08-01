use crate::{
  NonFungibleClassId,
  NonFungibleAssetId,
  FungibleAssetId,
  FungibleAssetBalance,
  DispatchResultAs,
  AttributeList, Locker, LockResult,
};
use sp_runtime::DispatchResult;

/// Trait for providing an interface to a non-fungible assets instances.
pub trait NonFungibleAssets<AccountId, Index> {

  fn mint_into(
    class_id: &NonFungibleClassId,
    who: &AccountId
  ) -> DispatchResultAs<NonFungibleAssetId>;

  /// Returns offer by given id
  fn get_offer(
    class_id: &NonFungibleClassId,
    offer_id: &u32,
  ) -> DispatchResultAs<(FungibleAssetId, FungibleAssetBalance, AttributeList)>;
  
	/// Assigns an attributes to asset  \
	/// The method doesn't check for the existance of either the class or the asset
  fn set_attributes(
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
    attributes: AttributeList,
  ) -> DispatchResult;

  /// Attempt to block an asset from the specified origin
  fn try_lock(
    who: &AccountId,
    origin: Locker<AccountId, Index>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> DispatchResultAs<LockResult>;
}
