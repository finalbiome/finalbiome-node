use super::*;

use crate::{
	mock::*, Error,
	ClassDetailsBuilder,
	Event as NfaEvent
};
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};

fn get_next_class_id() -> u32 {
	NextClassId::<Test>::get()
}

#[test]
fn template_test() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn next_class_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 0);
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 1);
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 2);
		assert_eq!(NextClassId::<Test>::get(), 3);
	});
}

#[test]
fn next_asset_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 0);
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 1);
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 2);
		assert_eq!(NextAssetId::<Test>::get(), 3);
	});
}

#[test]
fn class_details_builder() {
	new_test_ext().execute_with(|| {
		let b = ClassDetailsBuilder::<Test>::new(1, br"n2345678".to_vec()).unwrap(); // max 8 symbols
		let d = b.build().unwrap();
		assert_eq!(d.name.to_vec(), br"n2345678".to_vec());
		assert_eq!(d.owner, 1);
		assert_eq!(d.instances, 0);
		assert_eq!(d.bettor, None);
	});
}

#[test]
fn class_details_builder_name_len_exceed() {
	new_test_ext().execute_with(|| {
		assert_noop!(ClassDetailsBuilder::<Test>::new(1, br"n234567810".to_vec()), Error::<Test>::ClassNameTooLong); // max 8 symbols
	});
}

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(NonFungibleAssets::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(NonFungibleAssets::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(NonFungibleAssets::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn create_class_unsigned() {
	new_test_ext().execute_with(|| {
		let name = br"nfa name".to_vec();
		assert_noop!(NonFungibleAssets::create(Origin::none(), 2, name), sp_runtime::traits::BadOrigin);
	});
}
#[test]
fn create_class_created() {
	new_test_ext().execute_with(|| {
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		let nfa = Classes::<Test>::get(nfa_id).unwrap();
		assert_eq!(nfa.name.to_vec(), name);
		assert_eq!(nfa.instances, 0);
		assert_eq!(nfa.owner, org);
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), true);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::Created { class_id: nfa_id, owner: org }.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_destroy_class_unknown_class() {
	new_test_ext().execute_with(|| {
		assert_noop!(NonFungibleAssets::do_destroy_class(888, Some(999)), Error::<Test>::UnknownClass);
	});
}

#[test]
fn do_destroy_class_not_owner() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		assert_noop!(NonFungibleAssets::do_destroy_class(nfa_id, Some(3)), Error::<Test>::NoPermission);
		assert_eq!(Classes::<Test>::contains_key(nfa_id), true);
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), true);
	});
}

#[test]
fn do_destroy_class_worked() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		System::reset_events();
		assert_ok!(NonFungibleAssets::do_destroy_class(nfa_id, Some(org)));
		assert_eq!(Classes::<Test>::contains_key(nfa_id), false);
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), false);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::Destroyed { class_id: nfa_id }.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn destroy_class_not_org() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		System::reset_events();
		assert_noop!(NonFungibleAssets::destroy(Origin::none(), org, nfa_id), sp_runtime::traits::BadOrigin);
	});
}

#[test]
fn bettor_empty() {
	new_test_ext().execute_with(|| {
		let b:Bettor<u32, u32, u32, ConstU32<8>> = Bettor {
			outcomes: vec![].try_into().expect("Outcomes vec too big"),
			winnings: vec![].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn bettor_prob_more_100() {
	new_test_ext().execute_with(|| {
		let b:Bettor<
			<Test as pallet::Config>::FungibleAssetId,
			u32, <Test as pallet::Config>::FungibleAssetBalance,
			BoundedVec<u8,<Test as pallet::Config>::BettorOutcomeNameLimit>> = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 233,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::FA(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn bettor_probs_less_100() {
	new_test_ext().execute_with(|| {
		let b:Bettor<
			<Test as pallet::Config>::FungibleAssetId,
			u32, <Test as pallet::Config>::FungibleAssetBalance,
			BoundedVec<u8,<Test as pallet::Config>::BettorOutcomeNameLimit>> = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::FA(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false);

		let b:Bettor<
			<Test as pallet::Config>::FungibleAssetId,
			u32, <Test as pallet::Config>::FungibleAssetBalance,
			BoundedVec<u8,<Test as pallet::Config>::BettorOutcomeNameLimit>> = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 100,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::FA(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), true);
	});
}

#[test]
fn bettor_wins_empty() {
	new_test_ext().execute_with(|| {
		let b:Bettor<
			<Test as pallet::Config>::FungibleAssetId,
			u32, <Test as pallet::Config>::FungibleAssetBalance,
			BoundedVec<u8,<Test as pallet::Config>::BettorOutcomeNameLimit>> = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				},
				BettorOutcome {
					name: br"out1".to_vec().try_into().expect("too long"),
					probability: 95,
				},
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false);

		let b:Bettor<
			<Test as pallet::Config>::FungibleAssetId,
			u32, <Test as pallet::Config>::FungibleAssetBalance,
			BoundedVec<u8,<Test as pallet::Config>::BettorOutcomeNameLimit>> = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				},
				BettorOutcome {
					name: br"out1".to_vec().try_into().expect("too long"),
					probability: 95,
				},
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::FA(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), true);
	});
}
