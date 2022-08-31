use pallet_support::Locker;

use super::*;

impl<T: pallet::Config> pallet_support::traits::NonFungibleAssets<AccountIdOf<T>, IndexOf<T>>
  for Pallet<T>
{
  fn mint_into(
    class_id: &NonFungibleClassId,
    who: &AccountIdOf<T>,
  ) -> DispatchResultAs<NonFungibleAssetId> {
    Self::do_mint(*class_id, who.clone())
  }

  fn burn(
    class_id: NonFungibleClassId,
    asset_id: NonFungibleAssetId,
    maybe_check_owner: Option<&T::AccountId>,
  ) -> DispatchResult {
    Self::do_burn(class_id, asset_id, maybe_check_owner)
  }

  fn get_offer(
    class_id: &NonFungibleClassId,
    offer_id: &u32,
  ) -> DispatchResultAs<(FungibleAssetId, FungibleAssetBalance, AttributeList)> {
    let details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
    if let Some(purchased) = details.purchased {
      if let Some(offer) = purchased
        .offers
        .into_inner()
        .get(usize::try_from(*offer_id).map_err(|_| Error::<T>::WrongParameter)?)
      {
        Ok((offer.fa, offer.price, offer.attributes.clone()))
      } else {
        Err(Error::<T>::WrongParameter.into())
      }
    } else {
      Err(Error::<T>::UnsupportedCharacteristic.into())
    }
  }

  fn set_attributes(asset_id: &NonFungibleAssetId, attributes: AttributeList) -> DispatchResult {
    Self::assign_attributes(asset_id, attributes)
  }

  fn try_lock(
    who: &AccountIdOf<T>,
    origin: Locker<AccountIdOf<T>, IndexOf<T>>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> DispatchResultAs<LockResultOf<T>> {
    Self::set_lock(who, origin, class_id, asset_id)
  }

  fn clear_lock(
    who: &AccountIdOf<T>,
    origin: &Locker<AccountIdOf<T>, IndexOf<T>>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> sp_runtime::DispatchResult {
    Self::unset_lock(who, origin, class_id, asset_id)
  }

  fn get_class(class_id: &NonFungibleClassId) -> DispatchResultAs<ClassDetailsOf<T>> {
    Self::get_class_details(class_id)
  }
}
