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
    who: &T::AccountId,
    class_id: &NonFungibleClassId,
    offer_id: &u32,
  ) -> DispatchResult {
    use pallet_support::NonFungibleAssets;
    use pallet_support::FungibleAssets;
    // checking availability of that mechanic for the nfa class
    let (fa, price) = T::NonFungibleAssets::get_offer(class_id, offer_id)?;
    // check fa balances
    let _conseq = T::FungibleAssets::can_withdraw(fa, who, price);

    // mint nfa
    // set attributes
    // withdraw
    Ok(())
  }
}
