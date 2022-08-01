use super::*;

impl<T: pallet::Config> pallet_support::traits::NonFungibleAssets<AccountIdOf<T>, IndexOf<T>> for Pallet<T> {

  fn mint_into(
    class_id: &NonFungibleAssetId,
    who: &AccountIdOf<T>
  ) -> DispatchResultAs<NonFungibleAssetId> {
    Self::do_mint(*class_id, who.clone())
  }

  fn get_offer(
    class_id: &NonFungibleAssetId,
    offer_id: &u32,
  ) -> DispatchResultAs<(FungibleAssetId, FungibleAssetBalance, AttributeList)> {
    let details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
    if let Some(purchased) = details.purchased {
      if let Some(offer) = purchased.offers.into_inner().get(usize::try_from(*offer_id).map_err(|_| Error::<T>::WrongParameter)?) {
        Ok((offer.fa, offer.price, offer.attributes.clone()))
      } else {
        Err(Error::<T>::WrongParameter.into())
      }
    } else {
      Err(Error::<T>::UnsupportedCharacteristic.into())
    }
  }

  fn set_attributes(
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
    attributes: AttributeList
  ) -> DispatchResult {
    Self::assign_attributes(class_id, asset_id, attributes)
  }

  fn try_lock(
    who: &AccountIdOf<T>,
    origin: pallet_support::Locker<AccountIdOf<T>, IndexOf<T>>,
    class_id: &pallet_support::NonFungibleClassId,
    asset_id: &pallet_support::NonFungibleAssetId,
  ) -> DispatchResultAs<LockResult> {
      Self::set_lock(who, origin, class_id, asset_id)
  }
}
