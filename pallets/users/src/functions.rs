//! Functions for the Users pallet.

use frame_support::{pallet_prelude::DispatchResult, ensure};

use super::*;

impl<T: Config> Pallet<T> {
  /// Ensure that the sender is the registrar.
  pub fn ensure_registrar(sender: T::AccountId) -> DispatchResult {
    ensure!(Self::registrar_key().map_or(false, |k| sender == k), Error::<T>::RequireRegistrar);
    Ok(())
  }
}
