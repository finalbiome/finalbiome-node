use crate::{
	mock::*,
	Event as MechanicsEvent,
	Error,
	Timeouts, MechanicId, MechanicData, Mechanics, BetResult, Mechanic, MechanicUpgradeData, MechanicUpgradePayload, MechanicUpgradeDataOf, EventMechanicStopReason, AssetAction, MechanicDetailsBuilder, EventMechanicResultData, EventMechanicResultDataBet,
};
use frame_support::{assert_noop, assert_ok, BoundedVec, };
use frame_system::{EventRecord, Phase};

use pallet_support::{
	MechanicIdOf,
	bettor::{BettorOutcome, Bettor, BettorWinning, DrawOutcomeResult, OutcomeResult},
	DefaultListLengthLimit,
	AssetCharacteristic, purchased::{Purchased, Offer}, ClassDetailsOf, types_nfa::ClassDetails, LockedAccet,
};

#[macro_export]
macro_rules! bvec {
	($str:tt) => {
		$str.to_vec().try_into().unwrap()
	};
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	};
}

#[test]
fn template_test() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn mechanic_id_from_account() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		let n = System::account_nonce(acc);
		System::inc_account_nonce(acc);
		let id = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		assert_eq!(acc, id.account_id);
		assert_eq!(n+1, id.nonce);
	});
}

#[test]
fn drop_mechanic_none_mechanic() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		System::inc_account_nonce(acc);
		System::set_block_number(2);
		let _b = System::block_number();
		
		let id = MechanicsModule::get_mechanic_id(&acc);

		assert_ok!(MechanicsModule::drop_mechanic(&id, AssetAction::Release));
	});
}

#[test]
fn drop_mechanic_with_timeout() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		System::inc_account_nonce(acc);
		System::set_block_number(2);
		
		let id = MechanicsModule::get_mechanic_id(&acc);
		let details = MechanicDetailsBuilder::build::<Test>(acc, MechanicData::BuyNfa);
		let timeout_key = details.get_tiomeout_strorage_key(id.nonce);
		Mechanics::<Test>::insert(&id.account_id, &id.nonce, details);
		Timeouts::<Test>::insert(timeout_key, ());

		assert_ok!(MechanicsModule::drop_mechanic(&id, AssetAction::Release));
		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key(timeout_key), false);
	});
}

#[test]
fn do_buy_nfa_unknown_asset() {
	new_test_ext().execute_with(|| {
		// can_withdraw
		// FA 333 - UnknownAsset
		// price 99999 - NoFunds
		// get_offer
		// class_id == 1 && offer_id == 2 - FA=333, price=500
		assert_noop!(MechanicsModule::do_buy_nfa(&1, &1, &2), sp_runtime::TokenError::UnknownAsset);
	});
}

#[test]
fn do_buy_nfa_no_funds() {
	new_test_ext().execute_with(|| {
		// can_withdraw
		// FA 333 - UnknownAsset
		// price 99999 - NoFunds
		// get_offer
		// class_id == 1 && offer_id == 2 - FA=333, price=500
		// class_id == 1 && offer_id == 3 - FA=1, price=10000
		// else - FA=5, price=100
		assert_noop!(MechanicsModule::do_buy_nfa(&1, &1, &3), sp_runtime::TokenError::NoFunds);
	});
}

#[test]
fn do_buy_nfa_worked() {
	new_test_ext().execute_with(|| {
		// can_withdraw
		// FA 333 - UnknownAsset
		// price 99999 - NoFunds
		// get_offer
		// class_id == 1 && offer_id == 2 - FA=333, price=500
		// class_id == 1 && offer_id == 3 - FA=1, price=10000
		// else - FA=5, price=100
		assert_ok!(MechanicsModule::do_buy_nfa(&1, &1, &1));
	});
}

#[test]
fn mechanic_details_default() {
	new_test_ext().execute_with(|| {
		let md = MechanicDetailsBuilder::build::<Test>(1, MechanicData::BuyNfa);
		assert_eq!(md.owner, 1);
		assert_eq!(md.timeout_id, 21);
		assert_eq!(md.locked.to_vec(), [].to_vec());
		assert_eq!(md.data, MechanicData::BuyNfa);
	});
}

#[test]
fn choose_variant_works() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		for i in 0..100u32 {
			System::set_block_number(i.into());
			for j in 1..255 {
				let v = MechanicsModule::choose_variant(&mechanic_id, j);
				assert_eq!(v, i % j);
				assert!(v < j);
			}		
		}
	});
}

#[test]
fn choose_outcome_works() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		let outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 10,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 90,
				result: OutcomeResult::Lose,
			},
		];

		for i in 0..100u32 {
			System::set_block_number(i.into());
			let c = MechanicsModule::choose_outcome(&mechanic_id, &outcomes);
			assert!(c == 0 || c == 1);
			assert!((i < 10 && c == 0) || (i > 9 && c == 1));

			// assert_eq!(v, i % j);
		}
	});
}

#[test]
fn choose_outcome_with_one_outcome() {
	// it can't be used in pracrice
	new_test_ext().execute_with(|| {
		let acc = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		let outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 100,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 0,
				result: OutcomeResult::Lose,
			},
		];

		for i in 0..100u32 {
			System::set_block_number(i.into());
			let c = MechanicsModule::choose_outcome(&mechanic_id, &outcomes);
			assert!(0 == c);
		}
	});
}

#[test]
fn choose_outcome_with_outcome_as_fraction() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		let outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];

		for i in 0..100u32 {
			System::set_block_number(i.into());
			let c = MechanicsModule::choose_outcome(&mechanic_id, &outcomes);
			assert!(i % 3 == c);
		}
	});
}

#[test]
fn add_bet_result_first_time() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&acc);
		let outcomes = [1];
		System::set_block_number(10);
		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);
		assert_ok!(MechanicsModule::add_bet_result(&mechanic_id, &outcomes));
		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), true);
		let md = Mechanics::<Test>::get(&mechanic_id.account_id, &mechanic_id.nonce).unwrap();
		match md.data {
			MechanicData::Bet(bet_data) => assert_eq!(bet_data.outcomes.into_inner(), outcomes.to_vec()),
			_ => assert!(false),
		}
		assert_eq!(Timeouts::<Test>::contains_key((&30, &mechanic_id.account_id, &mechanic_id.nonce)), true);

		// second time
		let outcomes = [1, 3];
		assert_ok!(MechanicsModule::add_bet_result(&mechanic_id, &outcomes));
		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), true);
		let md = Mechanics::<Test>::get(&mechanic_id.account_id, &mechanic_id.nonce).unwrap();
		match md.data {
			MechanicData::Bet(bet_data) => assert_eq!(bet_data.outcomes.into_inner(), outcomes.to_vec()),
			_ => assert!(false),
		}
		assert_eq!(Timeouts::<Test>::contains_key((&30, &mechanic_id.account_id, &mechanic_id.nonce)), true);

	});
}

#[test]
fn try_finalize_bet_works() {
	new_test_ext().execute_with(|| {
		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1];
		let total = 1;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_some());
		assert_eq!(r, Some(BetResult::Lost));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [0];
		let total = 1;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_some());
		assert_eq!(r, Some(BetResult::Won));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [0];
		let total = 2;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1];
		let total = 2;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1, 0];
		let total = 2;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [0];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1, 0];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1, 1];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [0, 0];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Won));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [0, 0, 1];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Won));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1, 0, 1];
		let total = 3;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

		// TODO: fix a bug when the total number of rounds is less than the number of outcomes

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
		];
		let outcomes = [1, 0, 1, 1];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [0, 1, 2, 2];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());
		
		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [0, 1, 1, 2];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [0, 1, 1, 2, 1];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [0, 1, 1, 2, 0];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert!(r.is_none());
		
		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [0, 0, 0, ];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Won));
		
		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [1, 1, 1, ];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

		let bettor_outcomes: BoundedVec<BettorOutcome, DefaultListLengthLimit> = bvec![
			BettorOutcome {
				name: bvec!(br"o1"),
				probability: 1,
				result: OutcomeResult::Win,
			},
			BettorOutcome {
				name: bvec!(br"o2"),
				probability: 1,
				result: OutcomeResult::Lose,
			},
			BettorOutcome {
				name: bvec!(br"o3"),
				probability: 1,
				result: OutcomeResult::Draw,
			},
		];
		let outcomes = [2, 2, 2, 1, 1];
		let total = 5;
		let r = MechanicsModule::try_finalize_bet(&outcomes, total, &bettor_outcomes);
		assert_eq!(r, Some(BetResult::Lost));

	});
}

#[test]
fn do_bet_result_processing_win_nfa() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&who);
		let _class_id = 22;
		let _asset_id = 33;
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(10)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Lose,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		let result = BetResult::Won;

		let some_outcomes = vec![1, 2, 3];
		
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_result_processing(&mechanic_id, &who, &bettor, result.clone(), some_outcomes));
		// should mint nfa(10,) burn nfa(22,33), drop mechanic, deposit event

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![1, 2, 3],
							result: result,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_bet_result_processing_lose() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&who);
		let _class_id = 23;
		let _asset_id = 34;
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(666)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Lose,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		let result = BetResult::Lost;
		let some_outcomes = vec![1, 2, 3];
		
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_result_processing(&mechanic_id, &who, &bettor, result.clone(), some_outcomes));
		// should burn nfa(23,34), drop mechanic, deposit event

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
							id: mechanic_id.nonce,
							owner: mechanic_id.account_id,
							result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
								outcomes: bvec![1, 2, 3],
								result: result,
							}))
						}.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_bet_result_processing_draw_win() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&who);
		let _class_id = 24;
		let _asset_id = 35;
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(11)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Win,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		let result = BetResult::Draw;

		let some_outcomes = vec![1, 2, 3];

		
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_result_processing(&mechanic_id, &who, &bettor, result.clone(), some_outcomes));
		// should min nfa(11,) burn nfa(24,35), drop mechanic, deposit event

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![1, 2, 3],
							result: BetResult::Won,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_bet_result_processing_draw_lose() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&who);
		let _class_id = 25;
		let _asset_id = 36;
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(12)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Lose,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		let result = BetResult::Draw;
		let some_outcomes = vec![1, 2, 3];
		
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_result_processing(&mechanic_id, &who, &bettor, result.clone(), some_outcomes));
		// should burn nfa(25,36), drop mechanic, deposit event

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![1, 2, 3],
							result: BetResult::Lost,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_bet_result_processing_draw_keep() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let mechanic_id: MechanicIdOf<Test> = MechanicId::<<Test as frame_system::Config>::AccountId, <Test as frame_system::Config>::Index>::from_account_id::<Test>(&who);
		let _class_id = 26;
		let _asset_id = 37;
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(13)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		let result = BetResult::Draw;
		let some_outcomes = vec![1, 2, 3];
		
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_result_processing(&mechanic_id, &who, &bettor, result.clone(), some_outcomes));
		// should drop mechanic, deposit event

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![1, 2, 3],
							result: result,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);

	});
}


#[test]
fn can_use_mechanic_none() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(13)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		
		let purchased: Purchased = Purchased {
			offers: bvec![
				Offer {
					attributes: bvec![],
					price: 100.into(),
					fa: 0.into(),
				}
			]
		};
		assert!(purchased.is_valid()); // purchased must be valid
		let class_details: ClassDetailsOf<Test> = ClassDetails {
			owner: 1,
			attributes: 0,
			instances: 0,
			name: bvec!(br"ss"),
			bettor: None,
			purchased: None,
		};

		let mechanic = Mechanic::Bet;
		assert_noop!(MechanicsModule::can_use_mechanic(&mechanic, &class_details), Error::<Test>::IncompatibleAsset);
		let mechanic = Mechanic::BuyNfa;
		assert_noop!(MechanicsModule::can_use_mechanic(&mechanic, &class_details), Error::<Test>::IncompatibleAsset);
	});
}

#[test]
fn can_use_mechanic_bet() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(13)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		
		let purchased: Purchased = Purchased {
			offers: bvec![
				Offer {
					attributes: bvec![],
					price: 100.into(),
					fa: 0.into(),
				}
			]
		};
		assert!(purchased.is_valid()); // purchased must be valid
		let class_details: ClassDetailsOf<Test> = ClassDetails {
			owner: 1,
			attributes: 0,
			instances: 0,
			name: bvec!(br"ss"),
			bettor: Some(bettor),
			purchased: None,
		};

		let mechanic = Mechanic::Bet;
		assert_ok!(MechanicsModule::can_use_mechanic(&mechanic, &class_details));
		let mechanic = Mechanic::BuyNfa;
		assert_noop!(MechanicsModule::can_use_mechanic(&mechanic, &class_details), Error::<Test>::IncompatibleAsset);
	});
}
#[test]
fn can_use_mechanic_purchased() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(13)
			],
			rounds: 2,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid
		
		let purchased: Purchased = Purchased {
			offers: bvec![
				Offer {
					attributes: bvec![],
					price: 100.into(),
					fa: 0.into(),
				}
			]
		};
		assert!(purchased.is_valid()); // purchased must be valid
		let class_details: ClassDetailsOf<Test> = ClassDetails {
			owner: 1,
			attributes: 0,
			instances: 0,
			name: bvec!(br"ss"),
			bettor: None,
			purchased: Some(purchased),
		};

		let mechanic = Mechanic::Bet;
		assert_noop!(MechanicsModule::can_use_mechanic(&mechanic, &class_details), Error::<Test>::IncompatibleAsset);
		let mechanic = Mechanic::BuyNfa;
		assert_ok!(MechanicsModule::can_use_mechanic(&mechanic, &class_details));
	});
}

#[test]
fn try_lock_works() {
	new_test_ext().execute_with(|| {
		let id: MechanicIdOf<Test> = MechanicId {
			account_id: 1,
			nonce: 2,
		};

		let asset_id: LockedAccet = LockedAccet::Nfa(1, 2);
		let mut locks = [asset_id.clone()].to_vec();

		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), false);
		let details = MechanicDetailsBuilder::build::<Test>(1, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&id.account_id, &id.nonce, details);
		assert_ok!(MechanicsModule::try_lock(&id, asset_id.clone()));
		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), true);

		let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
		assert_eq!(m.locked.to_vec(), locks);

		for i in 1..256 {
			let asset_id: LockedAccet = LockedAccet::Nfa(1 + i, 2 + i);
			locks.push(asset_id.clone());
			if i == 255 {
				assert_noop!(MechanicsModule::try_lock(&id, asset_id.clone()), Error::<Test>::AssetsExceedsAllowable);
			} else {
				assert_ok!(MechanicsModule::try_lock(&id, asset_id.clone()));
				let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
				assert_eq!(m.locked.to_vec(), locks);
			}
		}
	});
}


#[test]
fn clear_lock_works() {
	new_test_ext().execute_with(|| {
		let id: MechanicIdOf<Test> = MechanicId {
			account_id: 1,
			nonce: 2,
		};

		let asset_id: LockedAccet = LockedAccet::Nfa(1, 2);
		let asset_id_2: LockedAccet = LockedAccet::Nfa(2, 3);
		let locks = [asset_id.clone(), asset_id_2.clone()].to_vec();

		let details = MechanicDetailsBuilder::build::<Test>(1, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&id.account_id, &id.nonce, details);
		assert_ok!(MechanicsModule::try_lock(&id, asset_id.clone()));
		assert_ok!(MechanicsModule::try_lock(&id, asset_id_2.clone()));
		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), true);

		let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
		assert_eq!(m.locked.to_vec(), locks);

		// ignoring not existed asset
		let asset_id_3: LockedAccet = LockedAccet::Nfa(3, 4);
		assert_ok!(MechanicsModule::_clear_lock(&id, asset_id_3));

		// chreck wrong mechanic
		let id_2: MechanicIdOf<Test> = MechanicId {
			account_id: 2,
			nonce: 3,
		};
		assert_noop!(MechanicsModule::_clear_lock(&id_2, asset_id_2), Error::<Test>::MechanicsNotAvailable);

		assert_ok!(MechanicsModule::_clear_lock(&id, asset_id.clone()));
		let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
		assert_eq!(m.locked.to_vec(), [asset_id_2.clone()].to_vec());
		
	});
}

#[test]
fn try_lock_nfa_works() {
	new_test_ext().execute_with(|| {
		let id: MechanicIdOf<Test> = MechanicId {
			account_id: 1,
			nonce: 2,
		};
		let who = 1;
		let class_id = 2;
		let asset_id = 3;

		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), false);
		let details = MechanicDetailsBuilder::build::<Test>(1, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&id.account_id, &id.nonce, details);
		assert_ok!(MechanicsModule::try_lock_nfa(&id, &who, class_id, asset_id));
		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), true);

	});
}

#[test]
fn crear_lock_nfa_works() {
	new_test_ext().execute_with(|| {
		let id: MechanicIdOf<Test> = MechanicId {
			account_id: 1,
			nonce: 2,
		};
		let who = 1;
		let class_id = 4;
		let asset_id = 5;
		let locks = [LockedAccet::Nfa(4, 5)].to_vec();

		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), false);
		let details = MechanicDetailsBuilder::build::<Test>(1, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&id.account_id, &id.nonce, details);
		assert_ok!(MechanicsModule::try_lock_nfa(&id, &who, class_id, asset_id));
		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), true);

		let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
		assert_eq!(m.locked.to_vec(), locks);

		assert_ok!(MechanicsModule::_clear_lock_nfa(&id, &who, class_id, asset_id));
		let m = Mechanics::<Test>::get(&id.account_id, &id.nonce).unwrap();
		assert_eq!(m.locked.to_vec().len(), 0);

	});
}

#[test]
fn play_bet_round_single_round_win() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(14)
			],
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 112;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 8;
		let _asset_id = 8;
		let outcomes = Vec::new();
		System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
		System::reset_events();

		// should mint Nfa(14), drop mechanic, deposit event
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));

		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![0,],
							result: BetResult::Won,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn play_bet_round_single_round_lose() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(15)
			],
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 113;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 9;
		let _asset_id = 9;
		let outcomes = Vec::new();
		System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
		System::reset_events();

		// should mint Nfa(15), drop mechanic, deposit event
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));

		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![1,],
							result: BetResult::Lost,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn play_bet_round_single_round_draw_keep() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
				BettorOutcome {
					name: bvec!(br"o3"),
					probability: 1,
					result: OutcomeResult::Draw,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(17)
			],
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 114;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 31;
		let _asset_id = 31;
		let outcomes = Vec::new();
		System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
		System::reset_events();

		// should deposit event
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));

		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![2,],
							result: BetResult::Draw,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn play_bet_round_single_round_draw_lose() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
				BettorOutcome {
					name: bvec!(br"o3"),
					probability: 1,
					result: OutcomeResult::Draw,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(18)
			],
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Lose,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 115;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 32;
		let _asset_id = 32;
		let outcomes = Vec::new();
		System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
		System::reset_events();

		// should burn Nfa(18), deposit event
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));

		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![2,],
							result: BetResult::Lost,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn play_bet_round_single_round_draw_win() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
				BettorOutcome {
					name: bvec!(br"o3"),
					probability: 1,
					result: OutcomeResult::Draw,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(19)
			],
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Win,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 115;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 33;
		let _asset_id = 33;
		let outcomes = Vec::new();
		System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
		System::reset_events();

		// should burn Nfa(18), deposit event
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));

		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![2,],
							result: BetResult::Won,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn play_bet_round_three_rounds_win_at_second_round() {
	new_test_ext().execute_with(|| {
		let bettor: Bettor = Bettor {
			// with odd block numbers the bettor will lose
			// with even - win
			outcomes: bvec![
				BettorOutcome {
					name: bvec!(br"o1"),
					probability: 1,
					result: OutcomeResult::Win,
				},
				BettorOutcome {
					name: bvec!(br"o2"),
					probability: 1,
					result: OutcomeResult::Lose,
				},
			],
			winnings: bvec![
				BettorWinning::Nfa(16)
			],
			rounds: 3,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert!(bettor.is_valid()); // bettor must be valid

		let who = 114;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let mechanic_id = MechanicsModule::get_mechanic_id(&who);
		let _class_id = 30;
		let _asset_id = 30;
		let outcomes = Vec::new();
		System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
		System::reset_events();

		// at first round should only save result to mechanic
		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));
		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), true);
		let m = Mechanics::<Test>::get(&mechanic_id.account_id, &mechanic_id.nonce).unwrap();
		if let MechanicData::Bet(data) = m.data {
			assert_eq!(data.outcomes.to_vec(),[0].to_vec()); // first round was won
		} else {
			assert!(false)
		}
		// after first round mechanic must have timeout
		assert_eq!(Timeouts::<Test>::contains_key((m.timeout_id, &mechanic_id.account_id, &mechanic_id.nonce)), true);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Stopped{ id: mechanic_id.nonce, owner: mechanic_id.account_id, reason: EventMechanicStopReason::UpgradeNeeded }.into(),
					topics: vec![],
				},
			]
		);
		// at second round with one more wins should mint Nfa(16), drop mechanic, deposit event, clean timeout
		let outcomes = vec![0];
		System::set_block_number(2); // rnd(4) % total_outcomes(2) = 0; 0 = win
		System::reset_events();

		assert_ok!(MechanicsModule::play_bet_round(&who, mechanic_id.clone(), &bettor, outcomes));
		assert_eq!(Mechanics::<Test>::contains_key(&mechanic_id.account_id, &mechanic_id.nonce), false);

		// after second, t.e. final round, mechanic must clean a timeout
		assert_eq!(Timeouts::<Test>::contains_key((m.timeout_id, &mechanic_id.account_id, &mechanic_id.nonce)), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: mechanic_id.nonce,
						owner: mechanic_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![0, 0],
							result: BetResult::Won,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn do_bet_unexisted_bet_asset() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let id = MechanicsModule::get_mechanic_id(&who);
		
		let class_id = 5;
		let asset_id = 6;

		assert_noop!(MechanicsModule::do_bet(&who, &class_id, &asset_id), sp_runtime::DispatchError::Other("mock_error_asset_doesnt_exist"));

		assert_eq!(Mechanics::<Test>::contains_key(&id.account_id, &id.nonce), false);
	});
}

#[test]
fn do_bet_asset_not_bettor() {
	new_test_ext().execute_with(|| {

		let who = 222;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let inner_id = MechanicsModule::get_mechanic_id(&who);
		
		let class_id = 7;
		let asset_id = 7;

		assert_noop!(MechanicsModule::do_bet(&who, &class_id, &asset_id), Error::<Test>::IncompatibleAsset);

		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), false);
	});
}

#[test]
fn do_bet_asset_one_round_work() {
	new_test_ext().execute_with(|| {

		let who = 116;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let inner_id = MechanicsModule::get_mechanic_id(&who);
		
		let class_id = 34;
		let asset_id = 34;

		System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet(&who, &class_id, &asset_id));
		
		// should mint Nfa(20), drop mechanic, deposit event

		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key((22, &inner_id.account_id, &inner_id.nonce)), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: inner_id.nonce,
						owner: inner_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![0,],
							result: BetResult::Won,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn do_bet_next_round_two_rounds_work() {
	new_test_ext().execute_with(|| {

		let who = 117;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		
		let class_id = 35;
		let asset_id = 35;
		
		System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
		let inner_id = MechanicsModule::get_mechanic_id(&who);
		let timeout_id: <Test as frame_system::Config>::BlockNumber = inner_id.nonce as u64 + 21;
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet(&who, &class_id, &asset_id));
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Stopped{ id: inner_id.nonce, owner: inner_id.account_id, reason: EventMechanicStopReason::UpgradeNeeded }.into(),
					topics: vec![],
				},
			]
		);
		// at first round should save mechanic
		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), true);
		// and set the timeout
		assert_eq!(Timeouts::<Test>::contains_key((timeout_id, &inner_id.account_id, &inner_id.nonce)), true);

		System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
		System::reset_events();
		assert_ok!(MechanicsModule::do_bet_next_round(&who, inner_id.clone()));
		// final result = draw
		// should burn nfa, drop mechanic, deposit event
		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: inner_id.nonce,
						owner: inner_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![0, 1],
							result: BetResult::Lost,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn do_do_upgrade_bet_two_rounds_work() {
	new_test_ext().execute_with(|| {

		let who = 118;
		let _n = System::account_nonce(who);
		System::inc_account_nonce(who);
		let inner_id = MechanicsModule::get_mechanic_id(&who);
		let timeout_id: <Test as frame_system::Config>::BlockNumber = inner_id.nonce as u64 + 21;
		
		let class_id = 36;
		let asset_id = 36;

		System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
		System::reset_events();
		assert_ok!(MechanicsModule::exec_bet(Origin::signed(who), class_id, asset_id));
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Stopped{ id: inner_id.nonce, owner: inner_id.account_id, reason: EventMechanicStopReason::UpgradeNeeded }.into(),
					topics: vec![],
				},
			]
		);
		
		// at first round should save mechanic
		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), true);
		// and set the timeout
		assert_eq!(Timeouts::<Test>::contains_key((timeout_id, &inner_id.account_id, &inner_id.nonce)), true);

		// NEXT round by upgrade mechanic
		System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
		System::reset_events();
		let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
			mechanic_id: inner_id.clone(),
			payload: MechanicUpgradePayload::Bet,
		};
		assert_ok!(MechanicsModule::upgrade(Origin::signed(who), upgrage_data));
		// final result = draw
		// should burn nfa, drop mechanic, deposit event
		assert_eq!(Mechanics::<Test>::contains_key(&inner_id.account_id, &inner_id.nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key((timeout_id, &inner_id.account_id, &inner_id.nonce)), false);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: MechanicsEvent::Finished {
						id: inner_id.nonce,
						owner: inner_id.account_id,
						result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
							outcomes: bvec![0u32, 1],
							result: BetResult::Lost,
						}))
					}.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn do_upgrade_who_not_own_mechanic_id() {
	new_test_ext().execute_with(|| {
		let who = 119;
		let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
			mechanic_id: MechanicId { account_id: 2, nonce: 3 },
			payload: MechanicUpgradePayload::Bet,
		};
		assert_noop!(MechanicsModule::do_upgrade(&who, upgrage_data), Error::<Test>::NoPermission);
	});
}

#[test]
fn do_upgrade_mechanic_not_exist() {
	new_test_ext().execute_with(|| {
		let who = 120;
		let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
			mechanic_id: MechanicId { account_id: who, nonce: 3 },
			payload: MechanicUpgradePayload::Bet,
		};
		assert_noop!(MechanicsModule::do_upgrade(&who, upgrage_data), Error::<Test>::MechanicsNotAvailable);
	});
}

#[test]
fn do_upgrade_mechanic_wrong_owner() {
	new_test_ext().execute_with(|| {
		let who = 121;
		let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
			mechanic_id: MechanicId { account_id: who, nonce: 3 },
			payload: MechanicUpgradePayload::Bet,
		};
		let details = MechanicDetailsBuilder::build::<Test>(2, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&who, &3, details);
		assert_noop!(MechanicsModule::do_upgrade(&who, upgrage_data), Error::<Test>::NoPermission);
	});
}

#[test]
fn do_upgrade_mechanic_incompatible_data() {
	new_test_ext().execute_with(|| {
		let who = 121;
		let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
			mechanic_id: MechanicId { account_id: who, nonce: 3 },
			payload: MechanicUpgradePayload::Bet,
		};
		let details = MechanicDetailsBuilder::build::<Test>(who, MechanicData::BuyNfa);
		Mechanics::<Test>::insert(&who, &3, details);
		assert_noop!(MechanicsModule::do_upgrade(&who, upgrage_data), Error::<Test>::IncompatibleData);
	});
}

#[test]
fn process_mechanic_timeouts_dropped() {
	new_test_ext().execute_with(|| {
		let who = 122;
		let nonce = 3;
		let timeout_id = 15;
		// add mechanic with timeout
		let mut details = MechanicDetailsBuilder::build::<Test>(who, MechanicData::BuyNfa);
		details.timeout_id = timeout_id;
		Mechanics::<Test>::insert(&who, &nonce, details);
		// add timeout records
		Timeouts::<Test>::insert((
			&timeout_id,
			&who,
			&nonce
		), ());
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), true);

		System::set_block_number(2);
		assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 0));
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), true);

		System::set_block_number(timeout_id);
		assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 1));

		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), false);
		System::set_block_number(timeout_id + 1);
		assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 0));
	});
}

#[test]
fn process_mechanic_timeouts_lifecycle() {
	new_test_ext().execute_with(|| {
		let who = 122;
		let nonce = 3;
		let timeout_id = 15;
		let timeout_id_2 = 150;
		// add mechanic with timeout
		let mut details = MechanicDetailsBuilder::build::<Test>(who, MechanicData::BuyNfa);
		details.timeout_id = timeout_id;
		Mechanics::<Test>::insert(&who, &nonce, details);
		// add timeout records
		Timeouts::<Test>::insert((
			&timeout_id,
			&who,
			&nonce
		), ());
		// add mechanic with timeout 2
		let mut details_2 = MechanicDetailsBuilder::build::<Test>(who, MechanicData::BuyNfa);
		details_2.timeout_id = timeout_id_2;
		Mechanics::<Test>::insert(&33, &44, details_2);
		// add timeout records 2
		Timeouts::<Test>::insert((
			&timeout_id_2,
			&33,
			&44
		), ());
		// add mechanic with timeout 3
		let mut details_3 = MechanicDetailsBuilder::build::<Test>(who, MechanicData::BuyNfa);
		details_3.timeout_id = timeout_id;
		Mechanics::<Test>::insert(&who, &44, details_3);
		// add timeout records 3
		Timeouts::<Test>::insert((
			&timeout_id,
			&who,
			&44
		), ());
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), true);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), true);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id_2, &33, &44)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), true);
		
		run_to_block(timeout_id);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), false);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id, &who, &nonce)), false);
		assert_eq!(Mechanics::<Test>::contains_key(&who, &nonce), false);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id_2, &33, &44)), true);
		assert_eq!(Mechanics::<Test>::contains_key(&33, &44), true);

		run_to_block(timeout_id_2);
		assert_eq!(Timeouts::<Test>::contains_key((&timeout_id_2, &33, &44)), false);
		assert_eq!(Mechanics::<Test>::contains_key(&33, &44), false);
	});
}

