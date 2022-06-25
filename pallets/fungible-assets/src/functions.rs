//! Functions for the Fungible-assets pallet.

use super::*;

impl<T: Config> Pallet<T> {

 /// Generate next id for new asset
 pub(super) fn get_next_asset_id() -> Result<AssetId, DispatchError> {
		NextAssetId::<T>::try_mutate(|id| -> Result<AssetId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;
			Ok(current_id)
		})
	}

  pub(super) fn new_account(
    who: &T::AccountId,
    asset_details: &mut AssetDetails<T::AccountId, T::Balance, NameLimit<T>>,
    maybe_deposit: Option<T::Balance>,
  ) -> Result<ExistenceReason<T::Balance>, DispatchError> {
    let accounts = asset_details.accounts.checked_add(1).ok_or(ArithmeticError::Overflow)?;
    let result = if let Some(deposit) = maybe_deposit {
      ExistenceReason::DepositHeld(deposit)
    } else {
      frame_system::Pallet::<T>::inc_sufficients(who);
			ExistenceReason::Sufficient
    };
    asset_details.accounts = accounts;
    Ok(result)
  }

  /// Get the asset `id` balance of `who` if the asset-account exists.
	pub fn maybe_balance(
		id: AssetId,
		who: impl sp_std::borrow::Borrow<T::AccountId>,
	) -> Option<T::Balance> {
		Accounts::<T>::get(id, who.borrow()).map(|a| a.balance)
	}

  pub(super) fn can_increase(
		id: AssetId,
		who: &T::AccountId,
		amount: T::Balance,
	) -> DepositConsequence {
		let details = match Assets::<T>::get(id) {
			Some(details) => details,
			None => return DepositConsequence::UnknownAsset,
		};
		if details.supply.checked_add(&amount).is_none() {
			return DepositConsequence::Overflow
		}
		if let Some(balance) = Self::maybe_balance(id, who) {
			if balance.checked_add(&amount).is_none() {
				return DepositConsequence::Overflow
			}
		}
		DepositConsequence::Success
	}

  /// Increases the asset `id` balance of `beneficiary` by `amount`.
  pub(super) fn increase_balance(
    id: AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
  ) -> DispatchResult {
    if amount.is_zero() {
			return Ok(())
		}
    Self::can_increase(id, beneficiary, amount).into_result()?;
    Assets::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
      let details = maybe_details.as_mut().ok_or(TokenError::UnknownAsset)?;
      
      details.supply = details.supply.saturating_add(amount);
      
      Accounts::<T>::try_mutate(id, beneficiary, |maybe_account| -> DispatchResult {
        match maybe_account {
          Some(ref mut account) => {
						account.balance.saturating_accrue(amount);
					},
          maybe_account @ None => {
            *maybe_account = Some(
              AssetAccountOf::<T> {
                balance: amount,
                reason: Self::new_account(beneficiary, details, None)?,
              }
            );
          },
        }
        Ok(())
      })?;
      Ok(())
    })?;

    Self::deposit_event(Event::Issued {
			asset_id: id,
			owner: beneficiary.clone(),
			total_supply: amount,
		});

    Ok(())
  }
}
