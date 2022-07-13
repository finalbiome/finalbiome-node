use super::*;
use crate::{
	mock::*, Error, CupFA, TopUppedFA, AssetDetailsBuilder, ExistenceReason,
	Event as FaEvent
};

use frame_support::{assert_noop, assert_ok, };
use sp_runtime::{TokenError};

use frame_system::{EventRecord, Phase};

fn get_next_fa_id() -> u32 {
	NextAssetId::<Test>::get()
}

#[test]
fn check_test_genesis_data() {
	new_test_ext().execute_with(|| {
		// genesis includes two assets with 0 & 1 ids
		let fa0 = Assets::<Test>::get(0).unwrap();
		assert_eq!(fa0.accounts, 2);
		assert_eq!(fa0.owner, 2);
		assert_eq!(fa0.name.to_vec(), br"asset01".to_vec());
		assert_eq!(fa0.supply, 11_000);
		assert_eq!(fa0.top_upped, None);
		assert_eq!(fa0.cup_global, None);
		assert_eq!(fa0.cup_local, None);
		let fa1 = Assets::<Test>::get(1).unwrap();
		assert_eq!(fa1.accounts, 2);
		assert_eq!(fa1.owner, 2);
		assert_eq!(fa1.name.to_vec(), br"asset02".to_vec());
		assert_eq!(fa1.supply, 30); // 25 initial + 5 top upped when start
		assert_eq!(fa1.top_upped.unwrap().speed, 5);
		assert_eq!(fa1.cup_global, None);
		assert_eq!(fa1.cup_local.unwrap().amount, 20);

		// genesis includes two accounts
		let acc1 = Accounts::<Test>::get(1, 0).unwrap();
		let acc3 = Accounts::<Test>::get(3, 1).unwrap();
		let acc4 = Accounts::<Test>::get(4, 1).unwrap();
		assert_eq!(acc1.balance, 1_000);
		assert_eq!(acc3.balance, 20);
		assert_eq!(acc4.balance, 10); // initial 5 and 5 top upped

		// next fa id must be 2
		assert_eq!(get_next_fa_id(), 2);

		// TopUppedAssets should includes asset02
		let tua = TopUppedAssets::<Test>::get();
		assert_eq!(tua.len(), 1);
		assert_eq!(tua.contains(&1), true);
		// TopUpQueue should include acc 4
		let tuq = TopUpQueue::<Test>::get(&1, &4).unwrap();
		assert_eq!(tuq, TopUpConsequence::TopUp(5));
	})
}

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(FungibleAssets::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(42));
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
		System::reset_events();
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

		let stored_fa = Assets::<Test>::get(fa_id);
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

		let fa = Assets::<Test>::get(fa_id).unwrap();
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
		let fa = Assets::<Test>::get(fa_id).unwrap();
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
		let fa = Assets::<Test>::get(fa_id).unwrap();
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
		let fa = Assets::<Test>::get(fa_id).unwrap();
		assert_eq!(fa.top_upped, Some(TopUppedFA { speed: 20 }));
		
		assert_eq!(true, TopUppedAssets::<Test>::get().contains(&fa_id));
	})
}

#[test]
fn next_step_topup() {
	new_test_ext().execute_with(|| {
		let fa = AssetDetails::<u64, NameLimit<Test>> {
			accounts: 1,
			cup_global: None,
			name: br"fa name".to_vec().try_into().unwrap(),
			owner: 2,
			supply: 100,
			top_upped: Some(TopUppedFA {speed: 5}),
			cup_local: Some(CupFA {amount: 20}),
		};

		assert_eq!(fa.next_step_topup(10), TopUpConsequence::TopUp(5));
		assert_eq!(fa.next_step_topup(5), TopUpConsequence::TopUp(5));
		assert_eq!(fa.next_step_topup(15), TopUpConsequence::TopUpFinal(5));
		assert_eq!(fa.next_step_topup(18), TopUpConsequence::TopUpFinal(2));
		assert_eq!(fa.next_step_topup(20), TopUpConsequence::None);
		assert_eq!(fa.next_step_topup(0), TopUpConsequence::TopUp(5));

		let fa = AssetDetails::<u64, NameLimit<Test>> {
			accounts: 1,
			cup_global: None,
			name: br"fa name".to_vec().try_into().unwrap(),
			owner: 2,
			supply: 100,
			top_upped: None,
			cup_local: Some(CupFA {amount: 20}),
		};
		assert_eq!(fa.next_step_topup(10), TopUpConsequence::None);

		let fa = AssetDetails::<u64, NameLimit<Test>> {
			accounts: 1,
			cup_global: None,
			name: br"fa name".to_vec().try_into().unwrap(),
			owner: 2,
			supply: 100,
			top_upped: Some(TopUppedFA {speed: 5}),
			cup_local: Some(CupFA {amount: 3}),
		};
		assert_eq!(fa.next_step_topup(10), TopUpConsequence::None);

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
		let fa = Assets::<Test>::get(fa_id).unwrap();
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
		let fa = Assets::<Test>::get(fa_id).unwrap();
		assert_eq!(fa.supply, 100);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(1, fa_id).unwrap();
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
		let fa = Assets::<Test>::get(fa_id).unwrap();
		assert_eq!(fa.supply, 200);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(1, fa_id).unwrap();
		assert_eq!(acc.balance, 200);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
		// add the same fa to the same acc
		assert_ok!(
			FungibleAssets::increase_balance(fa_id-1, &1, 300)
		);
		let fa = Assets::<Test>::get(fa_id-1).unwrap();
		assert_eq!(fa.supply, 400);
		assert_eq!(fa.accounts, 1);
		let acc = Accounts::<Test>::get(1, fa_id-1).unwrap();
		assert_eq!(acc.balance, 400);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
		// add fa to other acc
		assert_ok!(
			FungibleAssets::increase_balance(fa_id-1, &3, 1000)
		);
		let fa = Assets::<Test>::get(fa_id-1).unwrap();
		assert_eq!(fa.supply, 1400);
		assert_eq!(fa.accounts, 2);
		let acc = Accounts::<Test>::get(3, fa_id-1).unwrap();
		assert_eq!(acc.balance, 1000);
		assert_eq!(acc.reason, ExistenceReason::Sufficient);
	})
}

#[test]
fn increase_balance_event() {
	new_test_ext().execute_with(|| {
		System::reset_events();
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

#[test]
fn prep_debit_unknown_asset() {
	new_test_ext().execute_with(|| {
		let id = 999;
		let target = 10;
		let amount = 100;
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			TokenError::UnknownAsset
		);
	})
}

#[test]
fn prep_debit_unknown_account() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 10;
		let amount = 100;
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Error::<Test>::NoAccount
		);
	})
}

#[test]
fn prep_debit_can_debit() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 100;
		let max_allowed = false;
		assert_eq!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Ok(100)
		);
	})
}

#[test]
fn prep_debit_cant_debit() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 10000;
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			TokenError::NoFunds
		);
	})
}

#[test]
fn prep_debit_cant_debit_more_than_supply() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 12_000; // total suply 11_000
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			ArithmeticError::Underflow
		);
	})
}

#[test]
fn prep_debit_can_debit_allowed_more_than_supply() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 12_000; // total suply 11_000
		let max_allowed = true;
		assert_eq!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Ok(1000)
		);
	})
}

#[test]
fn prep_debit_can_debit_allowed() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 2_000; // total suply 11_000
		let max_allowed = true;
		assert_eq!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Ok(1000)
		);
	})
}

#[test]
fn prep_debit_can_debit_allowed_2() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 1_000; // total suply 11_000
		let max_allowed = true;
		assert_eq!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Ok(1000)
		);
	})
}

#[test]
fn prep_debit_can_debit_allowed_3() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 100; // total suply 11_000
		let max_allowed = true;
		assert_eq!(
			FungibleAssets::prep_debit(id, &target, amount, max_allowed),
			Ok(100)
		);
	})
}

#[test]
fn decrease_balance_unknown_asset() {
	new_test_ext().execute_with(|| {
		let id = 20;
		let target = 1; // account has 1000 of fa 0
		let amount = 100; // total suply 11_000
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			TokenError::UnknownAsset
		);
	})
}

#[test]
fn decrease_balance_unknown_account() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 100;
		let amount = 100; // total suply 11_000
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			Error::<Test>::NoAccount
		);
	})
}

#[test]
fn decrease_balance_no_founds() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1_000 of fa 0
		let amount = 10_000; // total suply 11_000
		let max_allowed = false;
		assert_noop!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			TokenError::NoFunds
		);
	})
}

#[test]
fn decrease_balance_common() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1000 of fa 0
		let amount = 100; // total suply 11_000
		let max_allowed = false;
		let fa_sup = Assets::<Test>::get(id).unwrap().supply;
		let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			Ok(100)
		);
		assert_eq!(
			fa_sup - amount,
			Assets::<Test>::get(id).unwrap().supply
		);
		assert_eq!(
			acc_balance - amount,
			Accounts::<Test>::get(target, id).unwrap().balance
		);
	})
}

#[test]
fn decrease_balance_max_allowed() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1_000 of fa 0
		let amount = 100_000; // total suply 11_000
		let max_allowed = true;
		let fa_sup = Assets::<Test>::get(id).unwrap().supply;
		let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			Ok(acc_balance)
		);
		assert_eq!(
			fa_sup - acc_balance,
			Assets::<Test>::get(id).unwrap().supply
		);
		assert_eq!(
			Accounts::<Test>::get(target, id).unwrap().balance,
			0
		);
	})
}

#[test]
fn decrease_balance_max_allowed_2() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let target = 1; // account has 1_000 of fa 0
		let amount = 900; // total suply 11_000
		let max_allowed = true;
		let fa_sup = Assets::<Test>::get(id).unwrap().supply;
		let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			Ok(amount)
		);
		assert_eq!(
			fa_sup - amount,
			Assets::<Test>::get(id).unwrap().supply
		);
		assert_eq!(
			Accounts::<Test>::get(target, id).unwrap().balance,
			acc_balance - amount
		);
	})
}

#[test]
fn decrease_balance_topup_check() {
	new_test_ext().execute_with(|| {
		let id = 1; // asset with top up
		let target = 3; // account has 20 of fa 1 and topup speed 5
		let amount = 3; // total suply 20
		let max_allowed = false;
		let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
		
		assert_eq!(TopUpQueue::<Test>::get(id, target).is_none(), true);
		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
			Ok(amount)
		);
		assert_eq!(
			Accounts::<Test>::get(target, id).unwrap().balance,
			acc_balance - amount // 17
		);
		assert_eq!(TopUpQueue::<Test>::get(id, target).is_none(), false);
		assert_eq!(TopUpQueue::<Test>::get(id, target).unwrap(), TopUpConsequence::TopUpFinal(3));

		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, 15, max_allowed),
			Ok(15)
		);
		assert_eq!(TopUpQueue::<Test>::get(id, target).unwrap(), TopUpConsequence::TopUp(5));

		assert_eq!(
			FungibleAssets::decrease_balance(id, &target, 1000, true),
			Ok(2)
		);
		assert_eq!(TopUpQueue::<Test>::get(id, target).unwrap(), TopUpConsequence::TopUp(5));

	})
}

#[test]
fn decrease_balance_event() {
	new_test_ext().execute_with(|| {
		System::reset_events();
		let id = 0;
		let target = 1; // account has 1_000 of fa 0
		let amount = 900; // total suply 11_000
		let max_allowed = true;
		assert_ok!(
			FungibleAssets::decrease_balance(id, &target, amount, max_allowed)
		);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: FaEvent::Burned { asset_id: id, owner: target, balance: amount }.into(),
					topics: vec![],
				},
			]
		);
	})
}

#[test]
fn top_upped_asset_manipulations_test() {
		new_test_ext().execute_with(|| {
			let curr_len = TopUppedAssets::<Test>::get().len();
			assert_ok!(FungibleAssets::top_upped_asset_add(&33));
			assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
			assert_eq!(true, TopUppedAssets::<Test>::get().contains(&33));

			assert_ok!(FungibleAssets::top_upped_asset_add(&33));
			assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
			assert_eq!(true, TopUppedAssets::<Test>::get().contains(&33));

			let curr_len = TopUppedAssets::<Test>::get().len();
			assert_ok!(FungibleAssets::top_upped_asset_add(&34));
			assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
			assert_eq!(true, TopUppedAssets::<Test>::get().contains(&34));

			let curr_len = TopUppedAssets::<Test>::get().len();
			FungibleAssets::top_upped_asset_remove(&34);
			assert_eq!(curr_len - 1, TopUppedAssets::<Test>::get().len());
			assert_eq!(false, TopUppedAssets::<Test>::get().contains(&34));

			let curr_len = TopUppedAssets::<Test>::get().len();
			FungibleAssets::top_upped_asset_remove(&34);
			assert_eq!(curr_len, TopUppedAssets::<Test>::get().len());
			assert_eq!(false, TopUppedAssets::<Test>::get().contains(&34));

			let curr_len = TopUppedAssets::<Test>::get().len();
			FungibleAssets::top_upped_asset_remove(&33);
			assert_eq!(curr_len - 1, TopUppedAssets::<Test>::get().len());
			assert_eq!(false, TopUppedAssets::<Test>::get().contains(&33));
		})
}


#[test]
fn top_upped_asset_remove_from_queue() {
	new_test_ext().execute_with(|| {
		// in genesis we have one asset with topup
		assert_eq!(TopUppedAssets::<Test>::get().len(), 1);
		// add fake record to TopUpQueue and check removing
		assert_eq!(true, TopUppedAssets::<Test>::get().contains(&1));
		TopUpQueue::<Test>::insert(&1, &3, TopUpConsequence::TopUpFinal(10));

		assert_eq!(TopUpQueue::<Test>::get(&1, &3).is_some(), true);
		_ = TopUpQueue::<Test>::get(&1, &3).unwrap();
		FungibleAssets::top_upped_asset_remove(&1);
		assert_eq!(TopUpQueue::<Test>::get(&1, &3).is_none(), true);
	})
}

#[test]
fn process_top_upped_assets() {
	new_test_ext().execute_with(|| {
		// create several accounts with balances
		let id = 1; // it's top upped asset with speed of 5 and limit 20
		assert_ok!(FungibleAssets::increase_balance(id, &1000, 20));
		assert_ok!(FungibleAssets::increase_balance(id, &1100, 17));
		assert_ok!(FungibleAssets::increase_balance(id, &1200, 1));
		// add it to queue
		TopUpQueue::<Test>::insert(&id, &1100, TopUpConsequence::TopUpFinal(3));
		TopUpQueue::<Test>::insert(&id, &1200, TopUpConsequence::TopUp(15));

		assert_eq!(FungibleAssets::process_top_upped_assets(), 0);

		assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20);
		assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 16);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1100), false);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1200), true);
		assert_eq!(TopUpQueue::<Test>::get(&id, &1200).unwrap(), TopUpConsequence::TopUpFinal(4));
	})
}

#[test]
fn process_top_up_in_progress() {
	new_test_ext().execute_with(|| {
		// create several accounts with balances
		let id = 1; // it's top upped asset with speed of 5 and limit 20
		assert_ok!(FungibleAssets::increase_balance(id, &1100, 17));
		assert_ok!(FungibleAssets::increase_balance(id, &1200, 1));
		// add it to queue
		TopUpQueue::<Test>::insert(&id, &1100, TopUpConsequence::TopUpFinal(3));
		TopUpQueue::<Test>::insert(&id, &1200, TopUpConsequence::TopUp(15));

		run_to_block(System::block_number() + 1);

		assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20);
		assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 16);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1100), false);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1200), true);

		run_to_block(System::block_number() + 1);

		assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20);
		assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 20);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1100), false);
		assert_eq!(TopUpQueue::<Test>::contains_key(&id, &1200), false);

	})
}
