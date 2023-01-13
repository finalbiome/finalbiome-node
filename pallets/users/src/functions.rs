//! Functions for the Users pallet.

use super::*;
use frame_support::{
  ensure,
  pallet_prelude::DispatchResult,
  traits::{tokens::imbalance::Imbalance, ConstU32, Currency},
  weights::Weight,
  BoundedVec,
};

use sp_runtime::traits::{Get, Verify};

type PositiveImbalanceOf<T> =
  <<T as Config>::Currency as frame_support::traits::tokens::currency::Currency<
    <T as frame_system::Config>::AccountId,
  >>::PositiveImbalance;

impl<T: Config> Pallet<T> {
  /// Ensure that the sender is the registrar.
  pub(crate) fn ensure_registrar(sender: T::AccountId) -> DispatchResult {
    ensure!(
      Self::registrar_key().map_or(false, |k| sender == k),
      Error::<T>::RequireRegistrar
    );
    Ok(())
  }

  pub(crate) fn verify_signature(
    sender: &T::AccountId,
    raw_signature: BoundedVec<u8, ConstU32<64>>,
  ) -> DispatchResult {
    use sp_core::crypto::ByteArray;

    let registrar = Self::registrar_key().ok_or(Error::<T>::UnknownRegistrar)?;

    // The signed message is the public address of the caller
    let message = sp_core::sr25519::Public::from_slice(sender.as_ref())
      .map_err(|_| Error::<T>::InvalidSignature)?;

    let verified = match sp_core::sr25519::Signature::try_from(&raw_signature[..]) {
      Ok(signature) => match sp_core::sr25519::Public::from_slice(registrar.as_ref()) {
        Ok(signer) => signature.verify(&message[..], &signer),
        _ => false,
      },
      _ => false,
    };

    if !verified {
      return Err(Error::<T>::InvalidSignature.into());
    }

    Ok(())
  }

  /// Calculate current slot based on current block number.
  pub(crate) fn get_current_slot() -> T::BlockNumber {
    let now = frame_system::Pallet::<T>::block_number();
    now % T::NumberOfSlots::get()
  }

  /// Get acvite slot for quota restores.
  pub(crate) fn get_active_slot(now: T::BlockNumber) -> T::BlockNumber {
    now % T::RecoveryPeriod::get()
  }

  pub(crate) fn do_sign_up(target: T::AccountId) -> DispatchResult {
    // 1. Ensures that the account has not yet been registered
    ensure!(
      !SlotsLookup::<T>::contains_key(&target),
      Error::<T>::Registered
    );

    // 2. Places into the Slots the periodic restoration
    let slot = Self::get_current_slot();
    Self::push_to_slot(slot, &target)?;

    // 3. Adds to lookup
    SlotsLookup::<T>::insert(&target, slot);

    // 4. Charges the target account the starting amount
    let balance = T::Currency::issue(T::Capacity::get());
    T::Currency::resolve_creating(&target, balance);

    Ok(())
  }

  pub(crate) fn push_to_slot(slot: T::BlockNumber, target: &T::AccountId) -> DispatchResult {
    let mut accounts = Slots::<T>::get(slot);
    accounts
      .try_push(target.clone())
      .map_err(|_| Error::<T>::Exhausted)?;

    Slots::<T>::insert(slot, accounts);

    Ok(())
  }

  /// Process all account in the given slot and restore them quotas.
  pub(crate) fn service_quotas(slot: T::BlockNumber) -> Weight {
    use sp_runtime::traits::Saturating;
    let mut reads: Weight = 1;
    let mut writes: Weight = 0;

    if let Ok(accounts) = Slots::<T>::try_get(slot) {
      let capacity = T::Capacity::get();
      let mut total_imbalance = PositiveImbalanceOf::<T>::zero();

      // 1. Deposit required amount to each account
      for account in &accounts {
        let imbalance = Self::restore_quota(account, capacity);
        reads.saturating_accrue(1); // restore_quota takes one read always and read and write if restore occures
        if imbalance.peek() > PositiveImbalanceOf::<T>::zero().peek() {
          total_imbalance.subsume(imbalance);
          reads.saturating_accrue(1);
          writes.saturating_accrue(1);
        }
      }
      // 2. Update the total issuance
      if total_imbalance.peek() > PositiveImbalanceOf::<T>::zero().peek() {
        let v = total_imbalance.peek();
        drop(T::Currency::issue(v).offset(total_imbalance));
        reads.saturating_accrue(1);
        writes.saturating_accrue(1);
      };
    };

    T::DbWeight::get().reads_writes(reads, writes)
  }

  /// Restore the quota amount for given user acount.
  pub(crate) fn restore_quota(
    account: &T::AccountId,
    capacity: <T::Currency as Currency<T::AccountId>>::Balance,
  ) -> PositiveImbalanceOf<T> {
    let total = T::Currency::free_balance(account);

    if total < capacity {
      T::Currency::deposit_creating(account, capacity - total)
    } else {
      PositiveImbalanceOf::<T>::zero()
    }
  }
}
