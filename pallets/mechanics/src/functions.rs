//! Functions for the Mechnics pallet.

use super::*;

impl<T: Config> Pallet<T> {
  /// Makes initial preparing for creating mechanic.
  /// Gets the id of the mechanic and fixes the timeout, 
  pub fn init_mechanic(who: T::AccountId) -> MechanicId<T> {
    let id = MechanicId::<T>::from_account_id(who);
    let block_number =  <frame_system::Pallet<T>>::block_number();
    let life_time_block = T::MechanicsLifeTime::get() + block_number;
    // set a timeout for mechanic
    Timeouts::<T>::insert((
      &life_time_block,
      &id.account_id,
      &id.nonce
    ), ());
    id
  }
  /// Execute Mechanic `BuyNFA`
  pub fn do_buy_nfa(
    _target: NonFungibleClassId<T>,
  ) {
    // check fa balances
    // mint nfa
    // set attributes
    // withdraw
  }
}
