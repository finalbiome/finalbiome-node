//! Functions for the Non-Fungible-Assets pallet.

use pallet_support::Locker;

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
	) -> DispatchResultAs<NonFungibleAssetId> {
		let mut asset_id = 0u32;
		Classes::<T>::try_mutate(&class_id, |maybe_class_details| -> DispatchResult {
			let class_details = maybe_class_details.as_mut().ok_or(Error::<T>::UnknownClass)?;
			
			asset_id = Self::get_next_asset_id()?;
			
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
		
		Ok(asset_id)
	}

	/// Creates attribute for the asset class.  \
	/// Attributes can be created only for classes
	pub fn do_create_attribute(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
		attribute: Attribute,
	) -> DispatchResult {
		// Checks attribute input
		attribute.validate()?;
		// Check class existance
		let mut details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
		if let Some(check_owner) = maybe_check_owner {
			ensure!(details.owner == check_owner, Error::<T>::NoPermission);
		}
		let asset_id:Option<NonFungibleAssetId> = None;
		let key = (&class_id, &asset_id, &attribute.key);
		// Attribute must not exits
		if Attributes::<T>::contains_key(&key) {
			return Err(Error::<T>::AttributeAlreadyExists.into())
		}

		Attributes::<T>::insert(&key, &attribute.value);
		details.attributes.saturating_inc();
		Classes::<T>::insert(&class_id, &details);
		Self::deposit_event(Event::AttributeCreated { class_id, key: attribute.key, value:attribute.value });
		Ok(())
	}

	/// Removes attribute from the asset class.  
	pub fn do_remove_attribute(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
		attribute_name: AttributeKey,
	) -> DispatchResult {
		let mut details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;
		if let Some(check_owner) = maybe_check_owner {
			ensure!(details.owner == check_owner, Error::<T>::NoPermission);
		}
		let asset_id:Option<NonFungibleAssetId> = None;
		if Attributes::<T>::take((&class_id, &asset_id, &attribute_name)).is_some() {
			details.attributes.saturating_dec();
			Classes::<T>::insert(&class_id, &details);
			Self::deposit_event(Event::AttributeRemoved { class_id, key: attribute_name });
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

	/// Assigns an attributes to asset  \
	/// The method doesn't check for the existance of either the class or the asset
	pub fn assign_attributes(class_id: &NonFungibleClassId, asset_id: &NonFungibleAssetId, attributes: AttributeList) -> DispatchResult {
		for attr in attributes.iter() {
			Attributes::<T>::insert((class_id, Some(asset_id), &attr.key), attr.value.clone());
		}
		Ok(())
	}

	pub(crate) fn do_set_characteristic(
		class_id: NonFungibleClassId,
		maybe_check_owner: Option<T::AccountId>,
		characteristic: Characteristic,
	) -> DispatchResult {
		// TODO: add references management.
		// If references to FA or NFA are changed then references in assets must be changed
		let mut details = Classes::<T>::get(class_id).ok_or(Error::<T>::UnknownClass)?;

		if let Some(check_owner) = maybe_check_owner {
			ensure!(details.owner == check_owner, Error::<T>::NoPermission);
		}

		match characteristic {
			Characteristic::Bettor(bettor) => {
				if let Some(inner) = &bettor {
					AssetCharacteristic::<T>::ensure(inner)?;
				};
				details.bettor = bettor;
			},
			Characteristic::Purchased(purchased) => {
				if let Some(inner) = &purchased {
					AssetCharacteristic::<T>::ensure(inner)?;
				};
				details.purchased = purchased;
			},
		};

		Classes::<T>::insert(&class_id, &details);
		Self::deposit_event(Event::Updated { class_id });
		Ok(())
	}


	/// Set an asset lock
	pub(crate) fn set_lock(
		who: &AccountIdOf<T>,
    origin: Locker<AccountIdOf<T>, IndexOf<T>>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
	) -> DispatchResultAs<LockResult> {
		// unlock not allowed
		ensure!(origin != Locker::None, Error::<T>::Locked);

		let mut details = Assets::<T>::get(class_id, asset_id).ok_or(Error::<T>::UnknownAsset)?;
		// ownership check
		ensure!(&details.owner == who, Error::<T>::NoPermission);

		match details.locked {
			Locker::None => {
				details.locked = origin;
				Assets::<T>::insert(class_id, asset_id, details);
				Ok(LockResult::Locked)
			},
			_ if details.locked == origin => {
				Ok(LockResult::Already)
			},
			_ => {
				Err(Error::<T>::Locked.into())
			},
		}
	}
}
