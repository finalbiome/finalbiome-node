//! Functions for the Fungible-assets pallet.

use super::*;

impl<T: Config> Pallet<T> {

 /// Generate next id for new asset
  pub(super) fn get_next_asset_id() -> DispatchResultAs<AssetId> {
		NextAssetId::<T>::try_mutate(|id| -> DispatchResultAs<AssetId> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;
			Ok(current_id)
		})
	}

  /// Reads O(0), Writes(1)
  pub(super) fn new_account(
    who: &T::AccountId,
    asset_details: &mut AssetDetails<T::AccountId, NameLimit<T>>,
    maybe_deposit: Option<AssetBalance>,
  ) -> DispatchResultAs<ExistenceReason> {
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

  /// Get the asset `id` balance of `who` if the asset-account exists. \
  /// Reads O(1), Writes(0)
	pub fn maybe_balance(
		id: AssetId,
		who: impl sp_std::borrow::Borrow<T::AccountId>,
	) -> Option<AssetBalance> {
		Accounts::<T>::get(who.borrow(), id).map(|a| a.balance)
	}

  /// Reads O(1), Writes(0)
  pub(super) fn can_increase(
		id: AssetId,
		who: &T::AccountId,
		amount: AssetBalance,
	) -> DepositConsequence {
    use DepositConsequence::*;
		let details = match Assets::<T>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};
		if details.supply.checked_add(amount).is_none() {
			return Overflow
		}
		if let Some(balance) = Self::maybe_balance(id, who) {
			if balance.checked_add(amount).is_none() {
				return Overflow
			}
		}
		Success
	}

  pub(super) fn can_decrease(
    id: AssetId,
		who: &T::AccountId,
		amount: AssetBalance,
  ) ->  WithdrawConsequence<AssetBalance> {
    use WithdrawConsequence::*;
    let details = match Assets::<T>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};
    if details.supply.checked_sub(amount).is_none() {
			return Underflow
		}
    if let Some(balance) = Self::maybe_balance(id, who) {
      if balance.checked_sub(amount).is_none() {
        NoFunds
      } else {
        Success
      }
    } else {
      NoFunds
    }
  }

  /// Returns the amount which should be debt from target with respect to `max_allowed` flag.
  /// If it `true`, then returns all accessible funds but no more than needed amount.
  pub(super) fn prep_debit(
    id: AssetId,
		target: &T::AccountId,
		amount: AssetBalance,
    max_allowed: bool,
  ) -> DispatchResultAs<AssetBalance> {
    let _ = Assets::<T>::get(id).ok_or(TokenError::UnknownAsset)?;
    let account = Accounts::<T>::get(target, id ).ok_or(Error::<T>::NoAccount)?;
    let actual = if max_allowed {
      account.balance.min(amount)
    } else {
      amount
    };
    ensure!(max_allowed || actual >= amount, TokenError::NoFunds);
    Self::can_decrease(id, target, actual).into_result()?;
    Ok(actual)
  }

  /// Increases the asset `id` balance of `beneficiary` by `amount`.
  /// Reads O(3), Writes(2)
  pub(super) fn increase_balance(
    id: AssetId,
		beneficiary: &T::AccountId,
		amount: AssetBalance,
  ) -> DispatchResult {
    if amount.is_zero() {
			return Ok(())
		}
    Self::can_increase(id, beneficiary, amount).into_result()?;
    Assets::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
      let details = maybe_details.as_mut().ok_or(TokenError::UnknownAsset)?;
      
      details.supply = details.supply.saturating_add(amount);
      
      Accounts::<T>::try_mutate(beneficiary, id, |maybe_account| -> DispatchResult {
        match maybe_account {
          Some(ref mut account) => {
						account.balance.saturating_accrue(amount);
					},
          maybe_account @ None => {
            *maybe_account = Some(
              AssetAccount {
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

  /// Decreases the asset `id` balance of `target` by `amount`.
  pub(super) fn decrease_balance(
    id: AssetId,
		target: &T::AccountId,
		amount: AssetBalance,
    max_allowed: bool,
  ) -> DispatchResultAs<AssetBalance> {
    if amount.is_zero() {
			return Ok(amount)
		}
    let actual = Self::prep_debit(id, target, amount, max_allowed)?;

    let mut target_topup: TopUpConsequence = TopUpConsequence::None;

    Assets::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
      let details = maybe_details.as_mut().ok_or(TokenError::UnknownAsset)?;
      
      // REFACT: support supply data in the asset details will be make huge resource consuming.
      // Maybe need drop it or move to off-chain stats collector
      details.supply = details.supply.saturating_sub(actual);

      Accounts::<T>::try_mutate(target, id, |maybe_account| -> DispatchResult {
        let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;
        account.balance = account.balance.saturating_sub(actual);

        // Check if asset is top upped
        target_topup = details.next_step_topup(account.balance);

        *maybe_account = Some(account);
        Ok(())
      })?;
      Ok(())
    })?;
    // Put an account to the queue for top upped of the balance
    match target_topup {
      TopUpConsequence::TopUp(topup) => TopUpQueue::<T>::insert(&id, &target, TopUpConsequence::TopUp(topup)),
      TopUpConsequence::TopUpFinal(topup) => TopUpQueue::<T>::insert(&id, &target, TopUpConsequence::TopUpFinal(topup)),
      TopUpConsequence::None => (),
    };

    Self::deposit_event(Event::Burned { asset_id: id, owner: target.clone(), balance: actual });
    Ok(actual)
  }

  /// Adds asset to TopUppedAssets storage.  \
  /// It adds only unique ids  \
  /// WARN: method doesn't check characteristics of the asset.  
  pub fn top_upped_asset_add(
    id: &AssetId,
  ) -> DispatchResult {
    let mut current_topupped = match <TopUppedAssets<T>>::try_get() {
      Ok(curr) => curr,
      Err(_) => <WeakBoundedVec<AssetId, T::MaxTopUppedAssets>>::try_from(Vec::new()).map_err(|()| Error::<T>::MaxTopUppedAssetsReached)?,
    };
    // let mut tu = current_topupped.into_inner();
    if let Err(index) = current_topupped.binary_search(id) {
      match current_topupped.try_insert(index, *id) {
        Ok(_) => <TopUppedAssets<T>>::put(current_topupped),
        Err(_) => return Err(Error::<T>::MaxTopUppedAssetsReached.into())
      };
    };
    Ok(())
  }
  
  /// Removes asset from TopUppedAssets storage.
  pub fn top_upped_asset_remove(
    id: &AssetId,
  ) {
    let mut current_topupped = match <TopUppedAssets<T>>::try_get() {
      Ok(curr) => curr,
      Err(_) => return,
    };
    if let Ok(index) = current_topupped.binary_search(id) {
      current_topupped.remove(index);
      <TopUppedAssets<T>>::put(current_topupped);
      // remove all records for that asset in TopUpQueue (if exisit)
      <TopUpQueue<T>>::remove_prefix(&id, None);
    };
  }

  /// Top up all assets which have demand.
  pub fn process_top_upped_assets() -> Weight {
    let mut reads: Weight = 1;
    let mut writes: Weight = 0;
    // get all top upped assets
    let assets = match TopUppedAssets::<T>::try_get() {
      Ok(assets) => assets,
      Err(_) => return T::DbWeight::get().reads(reads),
    };

    // loop over all assets, retrieve accounts that have a demand for these assets
    // and replenish their balance
    let mut next_topup: Vec<(AssetId, T::AccountId, TopUpConsequence)> = Vec::new();
    for id in assets.iter() {
      reads.saturating_accrue(1);
      for (target, topup) in TopUpQueue::<T>::drain_prefix(&id) {
        match topup {
          TopUpConsequence::TopUpFinal(amount) => {
            Self::increase_balance(*id, &target, amount).unwrap();
            reads.saturating_accrue(3);
            writes.saturating_accrue(2);
          },
          TopUpConsequence::TopUp(amount) => {
            Self::increase_balance(*id, &target, amount).unwrap();
            // it's not final top up. So, calculates next topup amount and stores it in the queue
            let account = Accounts::<T>::get(&target, &id).unwrap();
            let details = Assets::<T>::get(&id).unwrap();
            let target_topup = details.next_step_topup(account.balance);
            next_topup.push((*id, target, target_topup));
            reads.saturating_accrue(5);
            writes.saturating_accrue(2);
          },
          TopUpConsequence:: None => (),
        }
      }
    }
    // add to queue all unfinished top ups
    for (id, target, topup) in next_topup {
      TopUpQueue::<T>::insert(&id, &target, topup);
      writes.saturating_accrue(1);
    };

    T::DbWeight::get().reads_writes(reads, writes)
  }
}
