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
    use pallet_support::traits::NonFungibleAssets;
    use pallet_support::traits::FungibleAssets;
    // checking availability of that mechanic for the nfa class
    let (fa, price, attributes) = T::NonFungibleAssets::get_offer(class_id, offer_id)?;
    // check fa balances
    T::FungibleAssets::can_withdraw(fa, who, price).into_result()?;
    // mint nfa
    let asset_id = T::NonFungibleAssets::mint_into(class_id, who)?;
    // set attributes
    T::NonFungibleAssets::set_attributes(class_id, &asset_id, attributes)?;
    // withdraw
    T::FungibleAssets::burn_from(fa, who, price)?;
    Ok(())
  }
}
