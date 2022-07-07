//! Functions for the Non-Fungible-Assets pallet.

use super::*;

impl<T: Config> Pallet<T> {
   /// Generate next id for new class
   pub fn get_next_class_id() -> Result<NonFungibleClassId, DispatchError> {
		NextClassId::<T>::try_mutate(|id| -> Result<NonFungibleClassId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(NonFungibleClassId::one()).ok_or(Error::<T>::NoAvailableClassId)?;
			Ok(current_id)
		})
	}
   /// Generate next id for new asset
   pub fn get_next_asset_id() -> Result<NonFungibleAssetId, DispatchError> {
		NextAssetId::<T>::try_mutate(|id| -> Result<NonFungibleAssetId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(NonFungibleClassId::one()).ok_or(Error::<T>::NoAvailableAssetId)?;
			Ok(current_id)
		})
	}

	/// Reads = 1, writes = 2
	pub fn do_destroy_class(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
	) -> DispatchResult {
		Classes::<T>::try_mutate_exists(class_id, |maybe_details| {
			let class_details = maybe_details.take().ok_or(Error::<T>::UnknownClass)?;
			if let Some(check_owner) = maybe_check_owner {
				ensure!(class_details.owner == check_owner, Error::<T>::NoPermission);
			}
			ClassAccounts::<T>::remove(&class_details.owner, &class_id);
			Self::deposit_event(Event::Destroyed { class_id });
			Ok(())
		})
	}

	pub fn do_mint(
		class_id: NonFungibleClassId,
		owner: T::AccountId,
	) -> DispatchResult {
		Classes::<T>::try_mutate(&class_id, |maybe_class_details| -> DispatchResult {
			let class_details = maybe_class_details.as_mut().ok_or(Error::<T>::UnknownClass)?;
			
			let asset_id = Self::get_next_asset_id()?;
			
			// TODO: make check - org or member of org can't mint nfa
			
			let instances =
				class_details.instances.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			class_details.instances = instances;

			Accounts::<T>::insert((&owner, &class_id, &asset_id), ());

			let asset_details = AssetDetailsBuilder::<T>::new(owner.clone())?
				.build()?;
			Assets::<T>::insert(&class_id, &asset_id, asset_details);
			
			Self::deposit_event(Event::Issued { class_id, asset_id, owner });
			Ok(())
		})?;
		
		Ok(())
	}
}
