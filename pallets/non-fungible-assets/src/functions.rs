//! Functions for the Non-Fungible-Assets pallet.

use super::*;

impl<T: Config> Pallet<T> {
   /// Generate next id for new class
   pub(super) fn get_next_class_id() -> Result<ClassId, DispatchError> {
		NextClassId::<T>::try_mutate(|id| -> Result<ClassId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
			Ok(current_id)
		})
	}
   /// Generate next id for new asset
   pub(super) fn get_next_asset_id() -> Result<AssetId, DispatchError> {
		NextAssetId::<T>::try_mutate(|id| -> Result<AssetId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;
			Ok(current_id)
		})
	}

}
