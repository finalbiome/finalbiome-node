use super::*;

impl<T: Config> support::FungibleAssets<AccountIdOf<T>> for Pallet<T> {
  type AssetId = AssetId;
  type Balance = T::Balance;

fn can_withdraw(
		asset: Self::AssetId,
		who: &AccountIdOf<T>,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
    Pallet::<T>::can_decrease(asset, who, amount)
  }

fn burn_from(
    asset: Self::AssetId,
    who: &<T as SystemConfig>::AccountId,
    amount: Self::Balance
  ) -> support::DispatchResult<Self::Balance> {
    Self::decrease_balance(asset, who, amount, false)
  }


}
