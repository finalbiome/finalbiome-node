use super::*;
use crate::{
	mock::*, Error, CupFA, TopUppedFA, AssetDetailsBuilder, ExistenceReason,
	Event as FaEvent
};

use frame_support::{assert_noop, assert_ok, };
use sp_runtime::{TokenError};

use frame_system::{EventRecord, Phase};

fn get_next_fa_id() -> u32 {
	FungibleAssets::next_asset_id()
}

#[test]
fn check_test_genesis_data() {
	new_test_ext().execute_with(|| {
		// genesis includes two assets with 0 & 1 ids
		let fa0 = FungibleAssets::assets(0).unwrap();
		assert_eq!(fa0.accounts, 1);
		assert_eq!(fa0.owner, 2);
		assert_eq!(fa0.name.to_vec(), br"asset01".to_vec());
		assert_eq!(fa0.supply, 1000);
		assert_eq!(fa0.top_upped, None);
		assert_eq!(fa0.cup_global, None);
		assert_eq!(fa0.cup_local, None);
		let fa1 = FungibleAssets::assets(1).unwrap();
		assert_eq!(fa1.accounts, 1);
		assert_eq!(fa1.owner, 2);
		assert_eq!(fa1.name.to_vec(), br"asset02".to_vec());
		assert_eq!(fa1.supply, 20);
		assert_eq!(fa1.top_upped.unwrap().speed, 5);
		assert_eq!(fa1.cup_global, None);
		assert_eq!(fa1.cup_local.unwrap().amount, 20);

		// genesis includes two accounts
		let acc1 = FungibleAssets::accounts(0, 1).unwrap();
		let acc3 = FungibleAssets::accounts(1, 3).unwrap();
		assert_eq!(acc1.balance, 1000);
		assert_eq!(acc3.balance, 20);

		// next fa id must be 2
		assert_eq!(get_next_fa_id(), 2);
	})
}

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
		let fa_id = get_next_fa_id();
		let res = FungibleAssets::create(
			Origin::signed(1),
			org_id,
			name.clone(),
			None,
			None,
			None,
		);
		assert_ok!(res);

		let stored_fa = FungibleAssets::assets(fa_id);
		let fa = stored_fa.unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_global, None);
		assert_eq!(fa.cup_local, None);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: FaEvent::Created { asset_id: fa_id, owner: org_id }.into(),
					topics: vec![],
				},
			]
		);

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
		let fa_id = get_next_fa_id();
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

		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_global, None);
		assert_eq!(fa.cup_local, Some(CupFA { amount: 10 }));
		// global cup
		let fa_id = get_next_fa_id();
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
		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.name.to_vec(), name);
		assert_eq!(fa.top_upped, None);
		assert_eq!(fa.cup_local, None);
		assert_eq!(fa.cup_global, Some(CupFA { amount: 100 }));
		// both cups
		let fa_id = get_next_fa_id();
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
		let fa = FungibleAssets::assets(fa_id).unwrap();
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

		let fa_id = get_next_fa_id();
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
		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.top_upped, Some(TopUppedFA { speed: 20 }));
	})
}

#[test]
fn new_account() {
	new_test_ext().execute_with(|| {
		let mut asser_details = AssetDetailsBuilder::<Test>::new(1, br"test".to_vec()).unwrap().build().unwrap();
		assert_eq!(FungibleAssets::new_account(
			&1,
			&mut asser_details,
			Some(100),
		), Ok(ExistenceReason::DepositHeld(100)));

		let mut asser_details = AssetDetailsBuilder::<Test>::new(1, br"test".to_vec()).unwrap().build().unwrap();
		assert_eq!(FungibleAssets::new_account(
			&1,
			&mut asser_details,
			None,
		), Ok(ExistenceReason::Sufficient));
	})
}

#[test]
fn increase_balance_unknown_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			FungibleAssets::increase_balance(33, &1, 100),
			TokenError::UnknownAsset
		);
	})
}

#[test]
fn increase_balance_zero() {
	new_test_ext().execute_with(|| {
		// create asset
		let fa_id = get_next_fa_id();
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			2,
			br"fa name".to_vec(),
			None,
			None,
			None,
		));
		assert_ok!(
			FungibleAssets::increase_balance(0, &1, 0)
		);
		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.supply, 0);
	})
}

#[test]
fn increase_balance_straight_forward() {
	new_test_ext().execute_with(|| {
		// create asset
		let fa_id = get_next_fa_id();
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			2,
			br"fa name".to_vec(),
			None,
			None,
			None,
		));
		assert_ok!(
			FungibleAssets::increase_balance(fa_id, &1, 100)
		);
		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.supply, 100);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(fa_id, 1).unwrap();
		assert_eq!(acc.balance, 100);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
		// add another one and deposit it for the same acc
		let fa_id = get_next_fa_id();
		assert_ok!(FungibleAssets::create(
			Origin::signed(1),
			2,
			br"fa name2".to_vec(),
			None,
			None,
			None,
		));
		assert_ok!(
			FungibleAssets::increase_balance(fa_id, &1, 200)
		);
		let fa = FungibleAssets::assets(fa_id).unwrap();
		assert_eq!(fa.supply, 200);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(fa_id, 1).unwrap();
		assert_eq!(acc.balance, 200);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
		// add the same fa to the same acc
		assert_ok!(
			FungibleAssets::increase_balance(fa_id-1, &1, 300)
		);
		let fa = FungibleAssets::assets(fa_id-1).unwrap();
		assert_eq!(fa.supply, 400);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(fa_id-1, 1).unwrap();
		assert_eq!(acc.balance, 400);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
		// add fa to other acc
		assert_ok!(
			FungibleAssets::increase_balance(fa_id-1, &3, 1000)
		);
		let fa = FungibleAssets::assets(fa_id-1).unwrap();
		assert_eq!(fa.supply, 1400);
		assert_eq!(fa.accounts, 2);
		let acc = Accounts::<Test>::get(fa_id-1, 3).unwrap();
		assert_eq!(acc.balance, 1000);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
	})
}

#[test]
fn increase_balance_event() {
	new_test_ext().execute_with(|| {
		let fa_id = 0;
		let acc_id = 99;
		assert_ok!(
			FungibleAssets::increase_balance(fa_id, &acc_id, 100)
		);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: SysEvent::NewAccount { account: acc_id }.into(),
					topics: vec![],
				},
				EventRecord {
					phase: Phase::Initialization,
					event: FaEvent::Issued { asset_id: fa_id, owner: acc_id, total_supply: 100 }.into(),
					topics: vec![],
				},
			]
		);
	})
}
