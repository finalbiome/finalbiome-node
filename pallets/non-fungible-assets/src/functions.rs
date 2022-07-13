//! Functions for the Non-Fungible-Assets pallet.

use super::*;

impl<T: Config> Pallet<T> {
   /// Generate next id for new class
   pub fn get_next_class_id() -> DispatchResultAs<NonFungibleClassId> {
		NextClassId::<T>::try_mutate(|id| -> DispatchResultAs<NonFungibleClassId> {
			let current_id = *id;
			*id = id.checked_add(NonFungibleClassId::one()).ok_or(Error::<T>::NoAvailableClassId)?;
			Ok(current_id)
		})
	}
   /// Generate next id for new asset
   pub fn get_next_asset_id() -> DispatchResultAs<NonFungibleAssetId> {
		NextAssetId::<T>::try_mutate(|id| -> DispatchResultAs<NonFungibleAssetId> {
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
			// Remove attributes for class and for all instances
			Attributes::<T>::remove_prefix((&class_id,), None);
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

	/// Creates attribute for the asset class.  \
	/// Attributes can be created only for classes
	pub fn do_create_attribute(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
		attribute_name: Vec<u8>,
		attribute_value: AttributeTypeRaw,
	) -> DispatchResult {
		// Checks attribute input
		let name: BoundedVec<u8, T::AttributeKeyLimit> = attribute_name.try_into().map_err(|_| Error::<T>::AttributeConversionError)?;
		let value = AttributeDetailsBuilder::<T>::new(attribute_value)?.build()?;
		// Check class existance
		let mut details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
		if let Some(check_owner) = maybe_check_owner {
			ensure!(details.owner == check_owner, Error::<T>::NoPermission);
		}
		let asset_id:Option<NonFungibleAssetId> = None;
		let key = (&class_id, &asset_id, &name);
		// Attribute must not exits
		if Attributes::<T>::contains_key(&key) {
			return Err(Error::<T>::AttributeAlreadyExists.into())
		}

		Attributes::<T>::insert(&key, &value);
		details.attributes.saturating_inc();
		Classes::<T>::insert(&class_id, &details);
		Self::deposit_event(Event::AttributeCreated { class_id, key: name, value });
		Ok(())
	}

	/// Removes attribute from the asset class.  
	pub fn do_remove_attribute(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
		attribute_name: Vec<u8>,
	) -> DispatchResult {
		let key: BoundedVec<u8, T::AttributeKeyLimit> = attribute_name.try_into().map_err(|_| Error::<T>::AttributeConversionError)?;
		let mut details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
		if let Some(check_owner) = maybe_check_owner {
			ensure!(details.owner == check_owner, Error::<T>::NoPermission);
		}
		let asset_id:Option<NonFungibleAssetId> = None;
		if Attributes::<T>::take((&class_id, &asset_id, &key)).is_some() {
			details.attributes.saturating_dec();
			Classes::<T>::insert(&class_id, &details);
			Self::deposit_event(Event::AttributeRemoved { class_id, key });
		}
		Ok(())
	}

	/// Returns class details by class id
	/// Can return UnknownClass Error
	pub fn get_class_details(
		class_id: &NonFungibleClassId,
	) -> DispatchResultAs<ClassDetailsOf<T>> {
		Classes::<T>::get(class_id).ok_or_else(|| Error::<T>::UnknownClass.into())
	}
}
