use super::*;

impl<T: Config> pallet_support::traits::FungibleAssets<AccountIdOf<T>> for Pallet<T> {
  fn can_withdraw(
    asset: AssetId,
    who: &AccountIdOf<T>,
    amount: AssetBalance,
  ) -> WithdrawConsequence<AssetBalance> {
    Pallet::<T>::can_decrease(asset, who, amount)
  }

  fn mint_into(asset: AssetId, who: &AccountIdOf<T>, amount: AssetBalance) -> DispatchResult {
    Self::do_mint(asset, who, amount)
  }

  fn burn_from(
    asset: AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: AssetBalance,
  ) -> DispatchResultAs<AssetBalance> {
    Self::decrease_balance(asset, who, amount, false)
  }

  /// Increment the references counter on an asset.
  fn inc_references(asset: &AssetId) -> DispatchResult {
    Self::inc_references(asset)
  }

  fn dec_references(asset: &AssetId) -> DispatchResult {
    Self::dec_references(asset)
  }
}
