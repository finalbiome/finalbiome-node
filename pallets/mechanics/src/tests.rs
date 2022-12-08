use crate::{
  mock::*, AssetAction, BetResult, Error, Event as MechanicsEvent, EventMechanicResultData,
  EventMechanicResultDataBet, EventMechanicStopReason, Mechanic, MechanicData,
  MechanicDetailsBuilder, MechanicId, MechanicUpgradeData, MechanicUpgradeDataOf,
  MechanicUpgradePayload, Mechanics, Timeouts,
};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::{EventRecord, Phase};

use pallet_support::{
  bettor::{Bettor, BettorOutcome, BettorWinning, DrawOutcomeResult, OutcomeResult},
  purchased::{Offer, Purchased},
  types_nfa::ClassDetails,
  AssetCharacteristic, ClassDetailsOf, DefaultListLengthLimit, LockedAccet, MechanicIdOf, GamerAccount,
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
  new_test_ext().execute_with(|| {});
}

#[test]
fn mechanic_id_from_account() {
  new_test_ext().execute_with(|| {
    let acc = 222;
    let org = 333;
    let n = System::account_nonce(acc);
    System::inc_account_nonce(acc);
    let id = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
    assert_eq!(acc, id.gamer_account.account_id);
    assert_eq!(org, id.gamer_account.organization_id);
    assert_eq!(n + 1, id.nonce);
  });
}

#[test]
fn drop_mechanic_none_mechanic() {
  new_test_ext().execute_with(|| {
    let acc = 222;
    let org = 333;
    System::inc_account_nonce(acc);
    System::set_block_number(2);
    let _b = System::block_number();

    let id = MechanicsModule::get_mechanic_id(&acc, &org);

    assert_ok!(MechanicsModule::drop_mechanic(&id, AssetAction::Release));
  });
}

#[test]
fn drop_mechanic_with_timeout() {
  new_test_ext().execute_with(|| {
    let acc = 222;
    let org = 333;
    let ga = GamerAccount { account_id: acc, organization_id: org };
    System::inc_account_nonce(acc);
    System::set_block_number(2);

    let id = MechanicsModule::get_mechanic_id(&acc, &org);
    let details = MechanicDetailsBuilder::build::<Test>(ga, MechanicData::BuyNfa);
    let timeout_key = details.get_tiomeout_strorage_key(id.nonce);
    Mechanics::<Test>::insert(&id.gamer_account, &id.nonce, details);
    Timeouts::<Test>::insert(timeout_key.clone(), ());

    assert_ok!(MechanicsModule::drop_mechanic(&id, AssetAction::Release));
    assert!(!Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
    assert!(!Timeouts::<Test>::contains_key(timeout_key));
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
    assert_noop!(
      MechanicsModule::do_buy_nfa(&1, &1.into(), &2),
      sp_runtime::TokenError::UnknownAsset
    );
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
    assert_noop!(
      MechanicsModule::do_buy_nfa(&1, &1.into(), &3),
      sp_runtime::TokenError::NoFunds
    );
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
    assert_ok!(MechanicsModule::do_buy_nfa(&1, &1.into(), &1));
  });
}

#[test]
fn mechanic_details_default() {
  new_test_ext().execute_with(|| {
    let ga = GamerAccount {account_id: 1, organization_id: 2};
    let md = MechanicDetailsBuilder::build::<Test>(ga.clone(), MechanicData::BuyNfa);
    assert_eq!(md.owner, ga);
    assert_eq!(md.timeout_id, 21);
    assert_eq!(md.locked.to_vec(), [].to_vec());
    assert_eq!(md.data, MechanicData::BuyNfa);
  });
}

#[test]
fn choose_variant_works() {
  new_test_ext().execute_with(|| {
    let acc = 222;
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
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
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
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
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
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
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
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
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&acc, &org);
    let outcomes = [1];
    System::set_block_number(10);
    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));
    let md0 = MechanicsModule::add_bet_result(&mechanic_id, &outcomes).unwrap();
    assert!(Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));
    let md = Mechanics::<Test>::get(&mechanic_id.gamer_account, &mechanic_id.nonce).unwrap();
    match md.clone().data {
      MechanicData::Bet(bet_data) => assert_eq!(bet_data.outcomes.into_inner(), outcomes.to_vec()),
      _ => unreachable!(),
    }
    assert!(Timeouts::<Test>::contains_key((
      &30,
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    )));
    assert_eq!(md0, md);

    // second time
    let outcomes = [1, 3];
    // assert_ok!(MechanicsModule::add_bet_result(&mechanic_id, &outcomes));
    let md0 = MechanicsModule::add_bet_result(&mechanic_id, &outcomes).unwrap();

    assert!(Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));
    let md = Mechanics::<Test>::get(&mechanic_id.gamer_account, &mechanic_id.nonce).unwrap();
    match md.clone().data {
      MechanicData::Bet(bet_data) => assert_eq!(bet_data.outcomes.into_inner(), outcomes.to_vec()),
      _ => unreachable!(),
    }
    assert!(Timeouts::<Test>::contains_key((
      &30,
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    )));
    assert_eq!(md0, md);
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
    let outcomes = [0, 0, 0];
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
    let outcomes = [1, 1, 1];
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
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&who, &org);
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
      winnings: bvec![BettorWinning::Nfa(10.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Lose,
    };
    assert!(bettor.is_valid()); // bettor must be valid
    let result = BetResult::Won;

    let some_outcomes = vec![1, 2, 3];

    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_result_processing(
      &mechanic_id,
      &who,
      &bettor,
      result.clone(),
      some_outcomes
    ));
    // should mint nfa(10,) burn nfa(22,33), drop mechanic, deposit event

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1, 2, 3],
            result,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_result_processing_lose() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&who, &org);
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
      winnings: bvec![BettorWinning::Nfa(666.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Lose,
    };
    assert!(bettor.is_valid()); // bettor must be valid
    let result = BetResult::Lost;
    let some_outcomes = vec![1, 2, 3];

    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_result_processing(
      &mechanic_id,
      &who,
      &bettor,
      result.clone(),
      some_outcomes
    ));
    // should burn nfa(23,34), drop mechanic, deposit event

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1, 2, 3],
            result,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_result_processing_draw_win() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&who, &org);
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
      winnings: bvec![BettorWinning::Nfa(11.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Win,
    };
    assert!(bettor.is_valid()); // bettor must be valid
    let result = BetResult::Draw;

    let some_outcomes = vec![1, 2, 3];

    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_result_processing(
      &mechanic_id,
      &who,
      &bettor,
      result,
      some_outcomes
    ));
    // should min nfa(11,) burn nfa(24,35), drop mechanic, deposit event

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1, 2, 3],
            result: BetResult::Won,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_result_processing_draw_lose() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&who, &org);
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
      winnings: bvec![BettorWinning::Nfa(12.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Lose,
    };
    assert!(bettor.is_valid()); // bettor must be valid
    let result = BetResult::Draw;
    let some_outcomes = vec![1, 2, 3];

    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_result_processing(
      &mechanic_id,
      &who,
      &bettor,
      result,
      some_outcomes
    ));
    // should burn nfa(25,36), drop mechanic, deposit event

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1, 2, 3],
            result: BetResult::Lost,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_result_processing_draw_keep() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 333;
    let mechanic_id: MechanicIdOf<Test> = MechanicId::<
      <Test as frame_system::Config>::AccountId,
      <Test as frame_system::Config>::Index,
    >::from_account_id::<Test>(&who, &org);
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
      winnings: bvec![BettorWinning::Nfa(13.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid
    let result = BetResult::Draw;
    let some_outcomes = vec![1, 2, 3];

    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_result_processing(
      &mechanic_id,
      &who,
      &bettor,
      result.clone(),
      some_outcomes
    ));
    // should drop mechanic, deposit event

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1, 2, 3],
            result,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(13.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let purchased: Purchased = Purchased {
      offers: bvec![Offer {
        attributes: bvec![],
        price: 100.into(),
        fa: 0.into(),
      }],
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
    assert_noop!(
      MechanicsModule::can_use_mechanic(&mechanic, &class_details),
      Error::<Test>::IncompatibleAsset
    );
    let mechanic = Mechanic::BuyNfa;
    assert_noop!(
      MechanicsModule::can_use_mechanic(&mechanic, &class_details),
      Error::<Test>::IncompatibleAsset
    );
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
      winnings: bvec![BettorWinning::Nfa(13.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let purchased: Purchased = Purchased {
      offers: bvec![Offer {
        attributes: bvec![],
        price: 100.into(),
        fa: 0.into(),
      }],
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
    assert_noop!(
      MechanicsModule::can_use_mechanic(&mechanic, &class_details),
      Error::<Test>::IncompatibleAsset
    );
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
      winnings: bvec![BettorWinning::Nfa(13.into())],
      rounds: 2,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let purchased: Purchased = Purchased {
      offers: bvec![Offer {
        attributes: bvec![],
        price: 100.into(),
        fa: 0.into(),
      }],
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
    assert_noop!(
      MechanicsModule::can_use_mechanic(&mechanic, &class_details),
      Error::<Test>::IncompatibleAsset
    );
    let mechanic = Mechanic::BuyNfa;
    assert_ok!(MechanicsModule::can_use_mechanic(&mechanic, &class_details));
  });
}

#[test]
fn try_lock_works() {
  new_test_ext().execute_with(|| {
    let id: MechanicIdOf<Test> = MechanicId {
      gamer_account: GamerAccount { account_id: 1, organization_id: 3 },
      nonce: 2,
    };

    let asset_id: LockedAccet = LockedAccet::Nfa(1.into(), 2.into());
    let mut locks = [asset_id].to_vec();

    assert!(!Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
    let ga = GamerAccount { account_id: 1, organization_id: 3 };
    let details = MechanicDetailsBuilder::build::<Test>(ga, MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&id.gamer_account, &id.nonce, details);
    assert_ok!(MechanicsModule::try_lock(&id, asset_id));
    assert!(Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));

    let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
    assert_eq!(m.locked.to_vec(), locks);

    for i in 1..256 {
      let asset_id: LockedAccet = LockedAccet::Nfa((1 + i).into(), (2 + i).into());
      locks.push(asset_id);
      if i == 255 {
        assert_noop!(
          MechanicsModule::try_lock(&id, asset_id),
          Error::<Test>::AssetsExceedsAllowable
        );
      } else {
        assert_ok!(MechanicsModule::try_lock(&id, asset_id));
        let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
        assert_eq!(m.locked.to_vec(), locks);
      }
    }
  });
}

#[test]
fn clear_lock_works() {
  new_test_ext().execute_with(|| {
    let id: MechanicIdOf<Test> = MechanicId {
      gamer_account: GamerAccount { account_id: 1, organization_id: 3 },
      nonce: 2,
    };

    let asset_id: LockedAccet = LockedAccet::Nfa(1.into(), 2.into());
    let asset_id_2: LockedAccet = LockedAccet::Nfa(2.into(), 3.into());
    let locks = [asset_id, asset_id_2].to_vec();

    let ga = GamerAccount { account_id: 1, organization_id: 3 };
    let details = MechanicDetailsBuilder::build::<Test>(ga, MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&id.gamer_account, &id.nonce, details);
    assert_ok!(MechanicsModule::try_lock(&id, asset_id));
    assert_ok!(MechanicsModule::try_lock(&id, asset_id_2));
    assert!(Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));

    let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
    assert_eq!(m.locked.to_vec(), locks);

    // ignoring not existed asset
    let asset_id_3: LockedAccet = LockedAccet::Nfa(3.into(), 4.into());
    assert_ok!(MechanicsModule::_clear_lock(&id, asset_id_3));

    // chreck wrong mechanic
    let id_2: MechanicIdOf<Test> = MechanicId {
      gamer_account: GamerAccount { account_id: 1, organization_id: 3 },
      nonce: 3,
    };
    assert_noop!(
      MechanicsModule::_clear_lock(&id_2, asset_id_2),
      Error::<Test>::MechanicsNotAvailable
    );

    assert_ok!(MechanicsModule::_clear_lock(&id, asset_id));
    let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
    assert_eq!(m.locked.to_vec(), [asset_id_2].to_vec());
  });
}

#[test]
fn try_lock_nfa_works() {
  new_test_ext().execute_with(|| {
    let id: MechanicIdOf<Test> = MechanicId {
      gamer_account: GamerAccount { account_id: 1, organization_id: 3 },
      nonce: 2,
    };
    let who = 1;
    let class_id = 2.into();
    let asset_id = 3.into();
    let ga = GamerAccount { account_id: 1, organization_id: 3 };

    assert!(!Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
    let details = MechanicDetailsBuilder::build::<Test>(ga, MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&id.gamer_account, &id.nonce, details);
    assert_ok!(MechanicsModule::try_lock_nfa(&id, &who, class_id, asset_id));
    assert!(Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
  });
}

#[test]
fn crear_lock_nfa_works() {
  new_test_ext().execute_with(|| {
    let id: MechanicIdOf<Test> = MechanicId {
      gamer_account: GamerAccount { account_id: 1, organization_id: 3 },
      nonce: 2,
    };
    let who = 1;
    let ga = GamerAccount { account_id: who, organization_id: 3 };

    let class_id = 4.into();
    let asset_id = 5.into();
    let locks = [LockedAccet::Nfa(4.into(), 5.into())].to_vec();
    
    assert!(!Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
    let details = MechanicDetailsBuilder::build::<Test>(ga, MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&id.gamer_account, &id.nonce, details);
    assert_ok!(MechanicsModule::try_lock_nfa(&id, &who, class_id, asset_id));
    assert!(Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));

    let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
    assert_eq!(m.locked.to_vec(), locks);

    assert_ok!(MechanicsModule::_clear_lock_nfa(
      &id, &who, class_id, asset_id
    ));
    let m = Mechanics::<Test>::get(&id.gamer_account, &id.nonce).unwrap();
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
      winnings: bvec![BettorWinning::Nfa(14.into())],
      rounds: 1,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 112;
    let org = 223;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 8;
    let _asset_id = 8;
    let outcomes = Vec::new();
    System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
    System::reset_events();

    // should mint Nfa(14), drop mechanic, deposit event
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));

    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![0,],
            result: BetResult::Won,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(15.into())],
      rounds: 1,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 113;
    let org = 224;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 9;
    let _asset_id = 9;
    let outcomes = Vec::new();
    System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
    System::reset_events();

    // should mint Nfa(15), drop mechanic, deposit event
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));

    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![1,],
            result: BetResult::Lost,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(17.into())],
      rounds: 1,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 114;
    let org = 225;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 31;
    let _asset_id = 31;
    let outcomes = Vec::new();
    System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
    System::reset_events();

    // should deposit event
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));

    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![2,],
            result: BetResult::Draw,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(18.into())],
      rounds: 1,
      draw_outcome: DrawOutcomeResult::Lose,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 115;
    let org = 226;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 32;
    let _asset_id = 32;
    let outcomes = Vec::new();
    System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
    System::reset_events();

    // should burn Nfa(18), deposit event
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));

    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![2,],
            result: BetResult::Lost,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(19.into())],
      rounds: 1,
      draw_outcome: DrawOutcomeResult::Win,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 115;
    let org = 226;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 33;
    let _asset_id = 33;
    let outcomes = Vec::new();
    System::set_block_number(2); // rnd(2) % total_outcomes(3) = 2; 2 = draw
    System::reset_events();

    // should burn Nfa(18), deposit event
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));

    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![2,],
            result: BetResult::Won,
          }))
        }
        .into(),
        topics: vec![],
      },]
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
      winnings: bvec![BettorWinning::Nfa(16.into())],
      rounds: 3,
      draw_outcome: DrawOutcomeResult::Keep,
    };
    assert!(bettor.is_valid()); // bettor must be valid

    let who = 114;
    let org = 225;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let mechanic_id = MechanicsModule::get_mechanic_id(&who, &org);
    let _class_id = 30;
    let _asset_id = 30;
    let outcomes = Vec::new();
    System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
    System::reset_events();

    // at first round should only save result to mechanic
    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));
    assert!(Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));
    let m = Mechanics::<Test>::get(&mechanic_id.gamer_account, &mechanic_id.nonce).unwrap();
    if let MechanicData::Bet(data) = m.clone().data {
      assert_eq!(data.outcomes.to_vec(), [0].to_vec()); // first round was won
    } else {
      unreachable!()
    }
    // after first round mechanic must have timeout
    assert!(Timeouts::<Test>::contains_key((
      m.timeout_id,
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    )));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Stopped {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account.clone(),
          reason: EventMechanicStopReason::UpgradeNeeded(m.clone())
        }
        .into(),
        topics: vec![],
      },]
    );
    // at second round with one more wins should mint Nfa(16), drop mechanic, deposit event, clean
    // timeout
    let outcomes = vec![0];
    System::set_block_number(2); // rnd(4) % total_outcomes(2) = 0; 0 = win
    System::reset_events();

    assert_ok!(MechanicsModule::play_bet_round(
      &who,
      mechanic_id.clone(),
      &bettor,
      outcomes
    ));
    assert!(!Mechanics::<Test>::contains_key(
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    ));

    // after second, t.e. final round, mechanic must clean a timeout
    assert!(!Timeouts::<Test>::contains_key((
      m.timeout_id,
      &mechanic_id.gamer_account,
      &mechanic_id.nonce
    )));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: mechanic_id.nonce,
          owner: mechanic_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![0, 0],
            result: BetResult::Won,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_unexisted_bet_asset() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 333;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let id = MechanicsModule::get_mechanic_id(&who, &org);

    let class_id = 5.into();
    let asset_id = 6.into();

    assert_noop!(
      MechanicsModule::do_bet(&who, &org, &class_id, &asset_id),
      sp_runtime::DispatchError::Other("mock_error_asset_doesnt_exist")
    );

    assert!(!Mechanics::<Test>::contains_key(&id.gamer_account, &id.nonce));
  });
}

#[test]
fn do_bet_asset_not_bettor() {
  new_test_ext().execute_with(|| {
    let who = 222;
    let org = 222;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let inner_id = MechanicsModule::get_mechanic_id(&who, &org);

    let class_id = 7.into();
    let asset_id = 7.into();

    assert_noop!(
      MechanicsModule::do_bet(&who, &org, &class_id, &asset_id),
      Error::<Test>::IncompatibleAsset
    );

    assert!(!Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));
  });
}

#[test]
fn do_bet_asset_one_round_work() {
  new_test_ext().execute_with(|| {
    let who = 116;
    let org = 227;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let inner_id = MechanicsModule::get_mechanic_id(&who, &org);

    let class_id = 34.into();
    let asset_id = 34.into();

    System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
    System::reset_events();
    assert_ok!(MechanicsModule::do_bet(&who, &org, &class_id, &asset_id));

    // should mint Nfa(20), drop mechanic, deposit event

    assert!(!Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));
    assert!(!Timeouts::<Test>::contains_key((
      22,
      &inner_id.gamer_account,
      &inner_id.nonce
    )));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: inner_id.nonce,
          owner: inner_id.gamer_account,
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![0,],
            result: BetResult::Won,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_bet_next_round_two_rounds_work() {
  new_test_ext().execute_with(|| {
    let who = 117;
    let org = 228;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);

    let class_id = 35.into();
    let asset_id = 35.into();

    System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
    let inner_id = MechanicsModule::get_mechanic_id(&who, &org);
    let timeout_id: <Test as frame_system::Config>::BlockNumber = inner_id.nonce as u64 + 21;
    System::reset_events();
    assert_ok!(MechanicsModule::do_bet(&who, &org, &class_id, &asset_id));

    let m = Mechanics::<Test>::get(&inner_id.gamer_account, &inner_id.nonce).unwrap();
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Stopped {
          id: inner_id.nonce,
          owner: inner_id.gamer_account.clone(),
          reason: EventMechanicStopReason::UpgradeNeeded(m)
        }
        .into(),
        topics: vec![],
      },]
    );
    // at first round should save mechanic
    assert!(Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));
    // and set the timeout
    assert!(Timeouts::<Test>::contains_key((
      timeout_id,
      &inner_id.gamer_account,
      &inner_id.nonce
    )));

    System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
    System::reset_events();
    assert_ok!(MechanicsModule::do_bet_next_round(&who, inner_id.clone()));
    // final result = draw
    // should burn nfa, drop mechanic, deposit event
    assert!(!Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: inner_id.nonce,
          owner: inner_id.gamer_account.clone(),
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![0, 1],
            result: BetResult::Lost,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_do_upgrade_bet_two_rounds_work() {
  new_test_ext().execute_with(|| {
    let who = 118;
    let org = 229;
    let _n = System::account_nonce(who);
    System::inc_account_nonce(who);
    let inner_id = MechanicsModule::get_mechanic_id(&who, &org);
    let timeout_id: <Test as frame_system::Config>::BlockNumber = inner_id.nonce as u64 + 21;

    let class_id = 36.into();
    let asset_id = 36.into();

    System::set_block_number(2); // rnd(2) % total_outcomes(2) = 0; 0 = win
    System::reset_events();
    assert_ok!(MechanicsModule::exec_bet(
      Origin::signed(who),
      org,
      class_id,
      asset_id
    ));

    let m = Mechanics::<Test>::get(&inner_id.gamer_account, &inner_id.nonce).unwrap();
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Stopped {
          id: inner_id.nonce,
          owner: inner_id.gamer_account.clone(),
          reason: EventMechanicStopReason::UpgradeNeeded(m)
        }
        .into(),
        topics: vec![],
      },]
    );

    // at first round should save mechanic
    assert!(Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));
    // and set the timeout
    assert!(Timeouts::<Test>::contains_key((
      timeout_id,
      &inner_id.gamer_account,
      &inner_id.nonce
    )));

    // NEXT round by upgrade mechanic
    System::set_block_number(3); // rnd(3) % total_outcomes(2) = 1; 1 = lose
    System::reset_events();
    let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
      mechanic_id: inner_id.clone(),
      payload: MechanicUpgradePayload::Bet,
    };
    assert_ok!(MechanicsModule::upgrade(Origin::signed(who), org, upgrage_data));
    // final result = draw
    // should burn nfa, drop mechanic, deposit event
    assert!(!Mechanics::<Test>::contains_key(
      &inner_id.gamer_account,
      &inner_id.nonce
    ));
    assert!(!Timeouts::<Test>::contains_key((
      timeout_id,
      &inner_id.gamer_account,
      &inner_id.nonce
    )));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::Finished {
          id: inner_id.nonce,
          owner: inner_id.gamer_account.clone(),
          result: Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
            outcomes: bvec![0u32, 1],
            result: BetResult::Lost,
          }))
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_upgrade_who_not_own_mechanic_id() {
  new_test_ext().execute_with(|| {
    let who = 119;
    let org = 220;
    let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
      mechanic_id: MechanicId {
        gamer_account: GamerAccount { account_id: 2, organization_id: 4 },
        nonce: 3,
      },
      payload: MechanicUpgradePayload::Bet,
    };
    assert_noop!(
      MechanicsModule::do_upgrade(&who, &org, upgrage_data),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn do_upgrade_mechanic_not_exist() {
  new_test_ext().execute_with(|| {
    let who = 120;
    let org = 240;
    let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
      mechanic_id: MechanicId {
        gamer_account: GamerAccount { account_id: who, organization_id: org } ,
        nonce: 3,
      },
      payload: MechanicUpgradePayload::Bet,
    };
    assert_noop!(
      MechanicsModule::do_upgrade(&who, &org, upgrage_data),
      Error::<Test>::MechanicsNotAvailable
    );
  });
}

#[test]
fn do_upgrade_mechanic_wrong_owner() {
  new_test_ext().execute_with(|| {
    let who = 121;
    let org = 232;
    let ga = GamerAccount { account_id: who, organization_id: org };

    let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
      mechanic_id: MechanicId {
        gamer_account: ga.clone(),
        nonce: 3,
      },
      payload: MechanicUpgradePayload::Bet,
    };
    let ga2 = GamerAccount { account_id: 2, organization_id: org };
    let details = MechanicDetailsBuilder::build::<Test>(ga2, MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&ga, &3, details);
    assert_noop!(
      MechanicsModule::do_upgrade(&who, &org, upgrage_data),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn do_upgrade_mechanic_incompatible_data() {
  new_test_ext().execute_with(|| {
    let who = 121;
    let org = 232;
    let ga = GamerAccount { account_id: who, organization_id: org };

    let upgrage_data: MechanicUpgradeDataOf<Test> = MechanicUpgradeData {
      mechanic_id: MechanicId {
        gamer_account: GamerAccount { account_id: who, organization_id: org },
        nonce: 3,
      },
      payload: MechanicUpgradePayload::Bet,
    };
    let details = MechanicDetailsBuilder::build::<Test>(ga.clone(), MechanicData::BuyNfa);
    Mechanics::<Test>::insert(&ga, &3, details);
    assert_noop!(
      MechanicsModule::do_upgrade(&who, &org, upgrage_data),
      Error::<Test>::IncompatibleData
    );
  });
}

#[test]
fn process_mechanic_timeouts_dropped() {
  new_test_ext().execute_with(|| {
    let who = 122;
    let org = 233;
    let nonce = 3;
    let timeout_id = 15;
    let mid = GamerAccount { account_id: who, organization_id: org };
    // add mechanic with timeout
    let mut details = MechanicDetailsBuilder::build::<Test>(mid.clone(), MechanicData::BuyNfa);
    details.timeout_id = timeout_id;
    Mechanics::<Test>::insert(&mid, &nonce, details);
    // add timeout records
    Timeouts::<Test>::insert((&timeout_id, &mid, &nonce), ());
    assert!(Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(Mechanics::<Test>::contains_key(&mid, &nonce));

    System::set_block_number(2);
    assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 0));
    assert!(Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(Mechanics::<Test>::contains_key(&mid, &nonce));

    System::set_block_number(timeout_id);
    assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 1));
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: MechanicsEvent::DroppedByTimeout {
          owner: mid.clone(),
          id: nonce,
        }
        .into(),
        topics: vec![],
      },]
    );

    assert!(!Mechanics::<Test>::contains_key(&mid, &nonce));
    assert!(!Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    System::set_block_number(timeout_id + 1);
    assert_eq!(MechanicsModule::process_mechanic_timeouts(), (0, 0));
  });
}

#[test]
fn process_mechanic_timeouts_lifecycle() {
  new_test_ext().execute_with(|| {
    let who = 122;
    let org = 233;
    let nonce = 3;
    let timeout_id = 15;
    let timeout_id_2 = 150;
    // add mechanic with timeout
    let mid = GamerAccount { account_id: who, organization_id: org };

    let mut details = MechanicDetailsBuilder::build::<Test>(mid.clone(), MechanicData::BuyNfa);
    details.timeout_id = timeout_id;
    Mechanics::<Test>::insert(&mid, &nonce, details);
    // add timeout records
    Timeouts::<Test>::insert((&timeout_id, &mid, &nonce), ());
    // add mechanic with timeout 2
    let mut details_2 = MechanicDetailsBuilder::build::<Test>(mid.clone(), MechanicData::BuyNfa);
    details_2.timeout_id = timeout_id_2;
    let mid33 = GamerAccount { account_id: 33, organization_id: org };
    Mechanics::<Test>::insert(&mid33, &44, details_2);
    // add timeout records 2
    Timeouts::<Test>::insert((&timeout_id_2, &mid33, &44), ());
    // add mechanic with timeout 3
    let mut details_3 = MechanicDetailsBuilder::build::<Test>(mid.clone(), MechanicData::BuyNfa);
    details_3.timeout_id = timeout_id;
    Mechanics::<Test>::insert(&mid, &44, details_3);
    // add timeout records 3
    Timeouts::<Test>::insert((&timeout_id, &mid, &44), ());
    assert!(Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(Mechanics::<Test>::contains_key(&mid, &nonce));
    assert!(Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(Mechanics::<Test>::contains_key(&mid, &nonce));
    assert!(Timeouts::<Test>::contains_key((&timeout_id_2, &mid33, &44)));
    assert!(Mechanics::<Test>::contains_key(&mid, &nonce));

    run_to_block(timeout_id);
    assert!(!Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(!Mechanics::<Test>::contains_key(&mid, &nonce));
    assert!(!Timeouts::<Test>::contains_key((&timeout_id, &mid, &nonce)));
    assert!(!Mechanics::<Test>::contains_key(&mid, &nonce));
    assert!(Timeouts::<Test>::contains_key((&timeout_id_2, &mid33, &44)));
    assert!(Mechanics::<Test>::contains_key(&mid33, &44));

    run_to_block(timeout_id_2);
    assert!(!Timeouts::<Test>::contains_key((&timeout_id_2, &mid33, &44)));
    assert!(!Mechanics::<Test>::contains_key(&mid33, &44));
  });
}
