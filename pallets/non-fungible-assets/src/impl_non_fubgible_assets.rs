use super::*;

impl<T: pallet::Config> support::NonFungibleAssets<AccountIdOf<T>> for Pallet<T> {

fn mint_into(
    class_id: &NonFungibleAssetId,
    who: &AccountIdOf<T>
  ) -> DispatchResult {
    Self::do_mint(*class_id, who.clone())
  }

fn get_offer(
    class_id: &NonFungibleAssetId,
    offer_id: &u32,
  ) -> DispatchResultAs<(FungibleAssetId, FungibleAssetBalance)> {
        let details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
        if let Some(purchased) = details.purchased {
          if let Some(offer) = purchased.offers.into_inner().get(usize::try_from(*offer_id).map_err(|_| Error::<T>::WrongParameter)?) {
            Ok((offer.fa, offer.price))
          } else {
            Err(Error::<T>::WrongParameter.into())
          }
        } else {
          Err(Error::<T>::UnsupportedCharacreristic.into())
        }
    }
}
