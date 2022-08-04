use crate::{
  DispatchResultAs,
  FungibleAssetId,
  FungibleAssetBalance,
};
use frame_support::traits::tokens::WithdrawConsequence;
use sp_runtime::DispatchResult;

/// Trait for providing an interface to a fungible assets instances.
pub trait FungibleAssets<AccountId> {
  /// Returns `Failed` if the asset `balance` of `who` may not be decreased by `amount`, otherwise the consequence.
  fn can_withdraw(
		asset: FungibleAssetId,
		who: &AccountId,
		amount: FungibleAssetBalance,
	) -> WithdrawConsequence<FungibleAssetBalance>;
  /// Attempt to reduce the asset balance of who by amount.  \
  /// If not possible then don’t do anything. Possible reasons for failure include: \
  /// * Less funds in the account than amount
  /// * Liquidity requirements (locks, reservations) prevent the funds from being removed
  /// * Operation would require destroying the account and it is required to stay alive (e.g. because it’s providing a needed provider reference).
  /// 
  /// If successful it will reduce the overall supply of the underlying token.
  fn burn_from(
    asset: FungibleAssetId, 
    who: &AccountId, 
    amount: FungibleAssetBalance
  ) -> DispatchResultAs<FungibleAssetBalance>;

  /// Increment the references counter on an asset.
  fn inc_references(asset: &FungibleAssetId) -> DispatchResult;
  /// Increment the references counter on an asset.
  ///
	/// This *MUST* only be done once for every time you called `inc_references` on `who`.
  fn dec_references(asset: &FungibleAssetId) -> DispatchResult;

  /// Attempt to increase the `asset` balance of `who` by `amount`.
	///
	/// If not possible then don't do anything. Possible reasons for failure include:
	/// - Account cannot be created (e.g. because there is no provider reference and/or the asset
	///   isn't considered worth anything).
	///
	/// Since this is an operation which should be possible to take alone, if successful it will
	/// increase the overall supply of the underlying token.
	fn mint_into(asset: FungibleAssetId, who: &AccountId, amount: FungibleAssetBalance) -> DispatchResult;
}
