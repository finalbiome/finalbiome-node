//! Functions for the Organization Identity pallet.

use pallet_support::{
  traits::{FungibleAssets, NonFungibleAssets},
  AttributeList, FungibleAssetBalance, FungibleAssetId, NonFungibleClassId,
};

use super::*;

impl<T: Config> Pallet<T> {
  pub(crate) fn do_set_onboarding_assets(
    target: &T::AccountId,
    assets: OnboardingAssets,
  ) -> DispatchResultWithPostInfo {
    let mut details = Organizations::<T>::get(target).ok_or(Error::<T>::NotOrganization)?;
    details.onboarding_assets = assets;

    Organizations::<T>::insert(target, &details);
    Ok(().into())
  }

  pub(crate) fn do_onboarding(
    organization_id: &T::AccountId,
    target: &T::AccountId,
  ) -> DispatchResultWithPostInfo {
    let details = Organizations::<T>::get(organization_id).ok_or(Error::<T>::NotOrganization)?;

    if let Some(assets) = details.onboarding_assets {
      for asset in assets.into_iter() {
        match asset {
          AirDropAsset::Fa(asset_id, amount) => Self::do_airdrop_fa(target, asset_id, amount)?,
          AirDropAsset::Nfa(class_id, attributes) => {
            Self::do_airdrop_nfa(target, class_id, attributes)?
          },
        };
      }
    }

    // Set user as onboarded to the game
    UsersOf::<T>::insert(organization_id, target, ());

    Ok(().into())
  }

  pub(crate) fn do_airdrop_fa(
    who: &T::AccountId,
    asset_id: FungibleAssetId,
    amount: FungibleAssetBalance,
  ) -> DispatchResultWithPostInfo {
    T::FungibleAssets::mint_into(asset_id, who, amount)?;
    Ok(().into())
  }

  pub(crate) fn do_airdrop_nfa(
    target: &T::AccountId,
    class_id: NonFungibleClassId,
    attributes: AttributeList,
  ) -> DispatchResultWithPostInfo {
    // mint nfa
    let asset_id = T::NonFungibleAssets::mint_into(&class_id, target)?;
    // set attributes
    T::NonFungibleAssets::set_attributes(&asset_id, attributes)?;
    Ok(().into())
  }
}
