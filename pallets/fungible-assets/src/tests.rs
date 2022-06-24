use crate::{mock::*, Error, CupFA, TopUppedFA};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(FungibleAssets::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(FungibleAssets::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(FungibleAssets::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn create_fa_works() {
	new_test_ext().execute_with(|| {
		// Create fa with some name
		let name = br"fa name".to_vec();
		let org_id = 2;
		let res = FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			None,
			None,
		);
		assert_ok!(res);

		// Read pallet storage and assert an expected result.
		// let org = ensure_signed(Origin::signed(org_id)).unwrap();
		
		// TODO: change hardcoded asset id to extracting it from storage
		let stored_fa = FungibleAssets::assets(0);
		let fa = stored_fa.unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_global, None);
		assert_eq!(fa.cup_local, None);

		// TODO: test the events.
		//			 Impl bellow doesn't work
		// System::assert_has_event(Event::OrganizationIdentity(crate::Event::AddedToOrganization(name, org)));

		assert_noop!(FungibleAssets::create(
			Origin::none(),
			2,
			name.clone(),
			None,
			None,
			None,
		), sp_runtime::traits::BadOrigin);
	})
}

#[test]
fn create_fa_exceed_name_length() {
	new_test_ext().execute_with(|| {
		// Create fa with long name
		let name = br"some name012".to_vec();
		assert_noop!(FungibleAssets::create(
			Origin::signed(1),
			2,
			name,
			None,
			None,
			None,
		), Error::<Test>::AssetNameTooLong);
	})
}


#[test]
fn create_fa_with_cups() {
	new_test_ext().execute_with(|| {
		let name = br"fa name".to_vec();
		let org_id = 2;
		// local cup
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			None,
			Some(CupFA {
				amount: 10
			}),
		));
		// TODO: change hardcoded asset id to extracting it from storage
		let fa = FungibleAssets::assets(0).unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_global, None);
		assert_eq!(fa.cup_local, Some(CupFA { amount: 10 }));
		// global cup
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			Some(CupFA {
				amount: 100
			}),
			None,
		));
		// TODO: change hardcoded asset id to extracting it from storage
		let fa = FungibleAssets::assets(1).unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_local, None);
		assert_eq!(fa.cup_global, Some(CupFA { amount: 100 }));
		// both cups
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			Some(CupFA {
				amount: 100
			}),
			Some(CupFA {
				amount: 10
			}),
		));
		// TODO: change hardcoded asset id to extracting it from storage
		let fa = FungibleAssets::assets(2).unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_global, Some(CupFA { amount: 100 }));
		assert_eq!(fa.cup_local, Some(CupFA { amount: 10 }));

		// cups can't be set to zero
		assert_noop!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			Some(CupFA {
				amount: 0
			}),
			Some(CupFA {
				amount: 10
			}),
		), Error::<Test>::ZeroGlobalCup);
		assert_noop!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			Some(CupFA {
				amount: 10
			}),
			Some(CupFA {
				amount: 0
			}),
		), Error::<Test>::ZeroLocalCup);
	})
}

#[test]
fn create_fa_top_up() {
	new_test_ext().execute_with(|| {
		let name = br"fa name".to_vec();
		let org_id = 2;
		// can't set top up with no local cup
		assert_noop!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			Some(TopUppedFA {
				speed: 10
			}),
			Some(CupFA {
				amount: 10
			}),
			None,
		), Error::<Test>::TopUppedWithNoCup);

		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			Some(TopUppedFA {
				speed: 20
			}),
			Some(CupFA {
				amount: 100
			}),
			Some(CupFA {
				amount: 10
			}),
		));
		// TODO: change hardcoded asset id to extracting it from storage
		let fa = FungibleAssets::assets(0).unwrap();
		assert_eq!(fa.top_upped, Some(TopUppedFA { speed: 20 }));
	})
}
