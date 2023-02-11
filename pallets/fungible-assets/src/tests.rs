use super::*;
use crate::{
  mock::*, AssetDetailsBuilder, CupFA, Error, Event as FaEvent, ExistenceReason, TopUppedFA,
};

use frame_support::{assert_noop, assert_ok};
use pallet_support::FungibleAssetId;
use sp_runtime::TokenError;

use frame_system::{EventRecord, Phase};

fn get_next_fa_id() -> FungibleAssetId {
  NextAssetId::<Test>::get()
}

/// Helper which creates simple fa
fn create_fa(name: &str, org_id: u64) -> FungibleAssetId {
  let name = name.as_bytes().to_vec();
  let fa_id = get_next_fa_id();
  let res = FungibleAssets::create(Origin::signed(1), org_id, name, None, None, None);
  assert_ok!(res);
  fa_id
}

/// Helper which creates a top upped asset with speed of 5 and limit 20
fn create_topupped_fa(name: &str, org_id: u64) -> FungibleAssetId {
  let name = name.as_bytes().to_vec();
  let fa_id = get_next_fa_id();
  let res = FungibleAssets::create(
    Origin::signed(1),
    org_id,
    name.clone(),
    Some(TopUppedFA { speed: 5.into() }),
    None,
    Some(CupFA { amount: 20.into() }),
  );
  assert_ok!(res);
  fa_id
}

#[test]
fn create_fa_works() {
  new_test_ext().execute_with(|| {
    System::reset_events();
    // Create fa with some name
    let name = "fa name";
    let org_id = 2;
    let fa_id = create_fa(name, org_id);

    let stored_fa = Assets::<Test>::get(fa_id);
    let fa = stored_fa.unwrap();
    assert_eq!(fa.name.to_vec(), name.as_bytes().to_vec());
    assert_eq!(fa.top_upped, None);
    assert_eq!(fa.cup_global, None);
    assert_eq!(fa.cup_local, None);

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: FaEvent::Created {
          asset_id: fa_id,
          owner: org_id
        }
        .into(),
        topics: vec![],
      },]
    );

    assert_noop!(
      FungibleAssets::create(Origin::none(), 2, name.into(), None, None, None,),
      sp_runtime::traits::BadOrigin
    );
  })
}

#[test]
fn create_fa_exceed_name_length() {
  new_test_ext().execute_with(|| {
    // Create fa with long name
    let name = br"some name012".to_vec();
    assert_noop!(
      FungibleAssets::create(Origin::signed(1), 2, name, None, None, None,),
      Error::<Test>::AssetNameTooLong
    );
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
      Some(CupFA { amount: 10.into() }),
    ));

    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.name.to_vec(), name);
    assert_eq!(fa.top_upped, None);
    assert_eq!(fa.cup_global, None);
    assert_eq!(fa.cup_local, Some(CupFA { amount: 10.into() }));
    // global cup
    let fa_id = get_next_fa_id();
    assert_ok!(FungibleAssets::create(
      Origin::signed(1),
      org_id,
      name.clone(),
      None,
      Some(CupFA { amount: 100.into() }),
      None,
    ));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.name.to_vec(), name);
    assert_eq!(fa.top_upped, None);
    assert_eq!(fa.cup_local, None);
    assert_eq!(fa.cup_global, Some(CupFA { amount: 100.into() }));
    // both cups
    let fa_id = get_next_fa_id();
    assert_ok!(FungibleAssets::create(
      Origin::signed(1),
      org_id,
      name.clone(),
      None,
      Some(CupFA { amount: 100.into() }),
      Some(CupFA { amount: 10.into() }),
    ));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.name.to_vec(), name);
    assert_eq!(fa.top_upped, None);
    assert_eq!(fa.cup_global, Some(CupFA { amount: 100.into() }));
    assert_eq!(fa.cup_local, Some(CupFA { amount: 10.into() }));

    // cups can't be set to zero
    assert_noop!(
      FungibleAssets::create(
        Origin::signed(1),
        org_id,
        name.clone(),
        None,
        Some(CupFA { amount: 0.into() }),
        Some(CupFA { amount: 10.into() }),
      ),
      Error::<Test>::ZeroGlobalCup
    );
    assert_noop!(
      FungibleAssets::create(
        Origin::signed(1),
        org_id,
        name,
        None,
        Some(CupFA { amount: 10.into() }),
        Some(CupFA { amount: 0.into() }),
      ),
      Error::<Test>::ZeroLocalCup
    );
  })
}

#[test]
fn create_fa_top_up() {
  new_test_ext().execute_with(|| {
    let name = br"fa name".to_vec();
    let org_id = 2;
    // can't set top up with no local cup
    assert_noop!(
      FungibleAssets::create(
        Origin::signed(1),
        org_id,
        name.clone(),
        Some(TopUppedFA { speed: 10.into() }),
        Some(CupFA { amount: 10.into() }),
        None,
      ),
      Error::<Test>::TopUppedWithNoCup
    );

    let fa_id = get_next_fa_id();
    assert_ok!(FungibleAssets::create(
      Origin::signed(1),
      org_id,
      name,
      Some(TopUppedFA { speed: 20.into() }),
      Some(CupFA { amount: 100.into() }),
      Some(CupFA { amount: 10.into() }),
    ));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.top_upped, Some(TopUppedFA { speed: 20.into() }));

    assert!(TopUppedAssets::<Test>::get().contains(&fa_id));
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
      supply: 100.into(),
      references: 0,
      top_upped: Some(TopUppedFA { speed: 5.into() }),
      cup_local: Some(CupFA { amount: 20.into() }),
    };

    assert_eq!(
      fa.next_step_topup(10.into()),
      TopUpConsequence::TopUp(5.into())
    );
    assert_eq!(
      fa.next_step_topup(5.into()),
      TopUpConsequence::TopUp(5.into())
    );
    assert_eq!(
      fa.next_step_topup(15.into()),
      TopUpConsequence::TopUpFinal(5.into())
    );
    assert_eq!(
      fa.next_step_topup(18.into()),
      TopUpConsequence::TopUpFinal(2.into())
    );
    assert_eq!(fa.next_step_topup(20.into()), TopUpConsequence::None);
    assert_eq!(
      fa.next_step_topup(0.into()),
      TopUpConsequence::TopUp(5.into())
    );

    let fa = AssetDetails::<u64, NameLimit<Test>> {
      accounts: 1,
      cup_global: None,
      name: br"fa name".to_vec().try_into().unwrap(),
      owner: 2,
      supply: 100.into(),
      references: 0,
      top_upped: None,
      cup_local: Some(CupFA { amount: 20.into() }),
    };
    assert_eq!(fa.next_step_topup(10.into()), TopUpConsequence::None);

    let fa = AssetDetails::<u64, NameLimit<Test>> {
      accounts: 1,
      cup_global: None,
      name: br"fa name".to_vec().try_into().unwrap(),
      owner: 2,
      supply: 100.into(),
      references: 0,
      top_upped: Some(TopUppedFA { speed: 5.into() }),
      cup_local: Some(CupFA { amount: 3.into() }),
    };
    assert_eq!(fa.next_step_topup(10.into()), TopUpConsequence::None);
  })
}

#[test]
fn new_account() {
  new_test_ext().execute_with(|| {
    let mut asser_details = AssetDetailsBuilder::<Test>::new(1, br"test".to_vec())
      .unwrap()
      .build()
      .unwrap();
    assert_eq!(
      FungibleAssets::new_account(&1, &mut asser_details, Some(100.into()),),
      Ok(ExistenceReason::DepositHeld(100.into()))
    );

    let mut asser_details = AssetDetailsBuilder::<Test>::new(1, br"test".to_vec())
      .unwrap()
      .build()
      .unwrap();
    assert_eq!(
      FungibleAssets::new_account(&1, &mut asser_details, None,),
      Ok(ExistenceReason::Sufficient)
    );
  })
}

#[test]
fn increase_balance_unknown_asset() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      FungibleAssets::increase_balance(33.into(), &1, 100.into()),
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
    assert_ok!(FungibleAssets::increase_balance(0.into(), &1, 0.into()));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.supply, 0.into());
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
    assert_ok!(FungibleAssets::increase_balance(fa_id, &1, 100.into()));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.supply, 100.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id).unwrap();
    assert_eq!(acc.balance, 100.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add another one and deposit it for the same acc
    let fa_id_prev = fa_id;
    let fa_id = get_next_fa_id();
    assert_ok!(FungibleAssets::create(
      Origin::signed(1),
      2,
      br"fa name2".to_vec(),
      None,
      None,
      None,
    ));
    assert_ok!(FungibleAssets::increase_balance(fa_id, &1, 200.into()));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.supply, 200.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id).unwrap();
    assert_eq!(acc.balance, 200.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add the same fa to the same acc
    assert_ok!(FungibleAssets::increase_balance(fa_id_prev, &1, 300.into()));
    let fa = Assets::<Test>::get(fa_id_prev).unwrap();
    assert_eq!(fa.supply, 400.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id_prev).unwrap();
    assert_eq!(acc.balance, 400.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add fa to other acc
    assert_ok!(FungibleAssets::increase_balance(
      fa_id_prev,
      &3,
      1000.into()
    ));
    let fa = Assets::<Test>::get(fa_id_prev).unwrap();
    assert_eq!(fa.supply, 1400.into());
    assert_eq!(fa.accounts, 2);
    let acc = Accounts::<Test>::get(3, fa_id_prev).unwrap();
    assert_eq!(acc.balance, 1000.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
  })
}

#[test]
fn do_mint_straight_forward() {
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
    assert_ok!(FungibleAssets::do_mint(fa_id, &1, 100.into()));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.supply, 100.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id).unwrap();
    assert_eq!(acc.balance, 100.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add another one and deposit it for the same acc
    let fa_id_prev = fa_id;
    let fa_id = get_next_fa_id();
    assert_ok!(FungibleAssets::create(
      Origin::signed(1),
      2,
      br"fa name2".to_vec(),
      None,
      None,
      None,
    ));
    assert_ok!(FungibleAssets::do_mint(fa_id, &1, 200.into()));
    let fa = Assets::<Test>::get(fa_id).unwrap();
    assert_eq!(fa.supply, 200.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id).unwrap();
    assert_eq!(acc.balance, 200.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add the same fa to the same acc
    assert_ok!(FungibleAssets::do_mint(fa_id_prev, &1, 300.into()));
    let fa = Assets::<Test>::get(fa_id_prev).unwrap();
    assert_eq!(fa.supply, 400.into());
    assert_eq!(fa.accounts, 1);
    let acc = Accounts::<Test>::get(1, fa_id_prev).unwrap();
    assert_eq!(acc.balance, 400.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
    // add fa to other acc
    assert_ok!(FungibleAssets::do_mint(fa_id_prev, &3, 1000.into()));
    let fa = Assets::<Test>::get(fa_id_prev).unwrap();
    assert_eq!(fa.supply, 1400.into());
    assert_eq!(fa.accounts, 2);
    let acc = Accounts::<Test>::get(3, fa_id_prev).unwrap();
    assert_eq!(acc.balance, 1000.into());
    assert_eq!(acc.reason, ExistenceReason::Sufficient);
  })
}

#[test]
fn increase_balance_event() {
  new_test_ext().execute_with(|| {
    let fa_id = create_fa("fa name", 2);

    System::reset_events();
    let acc_id = 99;
    assert_ok!(FungibleAssets::increase_balance(fa_id, &acc_id, 100.into()));
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
          event: FaEvent::Issued {
            asset_id: fa_id,
            owner: acc_id,
            total_supply: 100.into()
          }
          .into(),
          topics: vec![],
        },
      ]
    );
  })
}

#[test]
fn do_mint_event() {
  new_test_ext().execute_with(|| {
    let fa_id = create_fa("fa name", 2);
    System::reset_events();
    let acc_id = 99;
    assert_ok!(FungibleAssets::do_mint(fa_id, &acc_id, 100.into()));
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
          event: FaEvent::Issued {
            asset_id: fa_id,
            owner: acc_id,
            total_supply: 100.into()
          }
          .into(),
          topics: vec![],
        },
      ]
    );
  })
}

#[test]
fn prep_debit_unknown_asset() {
  new_test_ext().execute_with(|| {
    let id = 999.into();
    let target = 10;
    let amount = 100;
    let max_allowed = false;
    assert_noop!(
      FungibleAssets::prep_debit(id, &target, amount.into(), max_allowed),
      TokenError::UnknownAsset
    );
  })
}

#[test]
fn prep_debit_unknown_account() {
  new_test_ext().execute_with(|| {
    let fa_id = create_fa("fa name", 2);

    let target = 10;
    let amount = 100.into();
    let max_allowed = false;
    assert_noop!(
      FungibleAssets::prep_debit(fa_id, &target, amount, max_allowed),
      Error::<Test>::NoAccount
    );
  })
}

#[test]
fn prep_debit_can_debit() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account must have 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    let amount = 100.into();
    let max_allowed = false;
    assert_eq!(
      FungibleAssets::prep_debit(id, &target, amount, max_allowed),
      Ok(100.into())
    );
  })
}

#[test]
fn prep_debit_cant_debit() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    assert_ok!(FungibleAssets::do_mint(id, &5, 10_000.into())); // workaround of arifmetic underflow

    let amount = 10000.into();
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
    // Create fa with some name
    let name = br"fa name".to_vec();
    let org_id = 2;
    let id = get_next_fa_id();
    let res = FungibleAssets::create(Origin::signed(1), org_id, name.clone(), None, None, None);
    assert_ok!(res);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    let amount = 12_000.into(); // total suply 11_000
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
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    let amount = 12_000.into(); // total suply 11_000
    let max_allowed = true;
    assert_eq!(
      FungibleAssets::prep_debit(id, &target, amount, max_allowed),
      Ok(1000.into())
    );
  })
}

#[test]
fn prep_debit_can_debit_allowed() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    let amount = 2_000.into(); // total suply 11_000
    let max_allowed = true;
    assert_eq!(
      FungibleAssets::prep_debit(id, &target, amount, max_allowed),
      Ok(1000.into())
    );
  })
}

#[test]
fn prep_debit_can_debit_allowed_2() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    let amount = 1_000.into(); // total suply 11_000
    let max_allowed = true;
    assert_eq!(
      FungibleAssets::prep_debit(id, &target, amount, max_allowed),
      Ok(1000.into())
    );
  })
}

#[test]
fn prep_debit_can_debit_allowed_3() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    let amount = 100.into(); // total suply 11_000
    let max_allowed = true;
    assert_eq!(
      FungibleAssets::prep_debit(id, &target, amount, max_allowed),
      Ok(100.into())
    );
  })
}

#[test]
fn decrease_balance_unknown_asset() {
  new_test_ext().execute_with(|| {
    let id = 20.into();
    let target = 1; // account has 1000 of fa 0
    let amount = 100.into(); // total suply 11_000
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
    let id = create_fa("fa name", 2);

    let target = 100;
    let amount = 100.into(); // total suply 11_000
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
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1_000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    assert_ok!(FungibleAssets::do_mint(id, &5, 10_000.into())); // workaround of arifmetic underflow

    let amount = 10_000.into(); // total suply 11_000
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
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    let amount = 100.into(); // total suply 11_000
    let max_allowed = false;
    let fa_sup = Assets::<Test>::get(id).unwrap().supply;
    let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
    assert_eq!(
      FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
      Ok(100.into())
    );
    assert_eq!(fa_sup - amount, Assets::<Test>::get(id).unwrap().supply);
    assert_eq!(
      acc_balance - amount,
      Accounts::<Test>::get(target, id).unwrap().balance
    );
  })
}

#[test]
fn decrease_balance_max_allowed() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1_000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    let amount = 100_000.into(); // total suply 11_000
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
    assert_eq!(Accounts::<Test>::get(target, id).unwrap().balance, 0.into());
  })
}

#[test]
fn decrease_balance_max_allowed_2() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1_000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));
    let amount = 900.into(); // total suply 11_000
    let max_allowed = true;
    let fa_sup = Assets::<Test>::get(id).unwrap().supply;
    let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;
    assert_eq!(
      FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
      Ok(amount)
    );
    assert_eq!(fa_sup - amount, Assets::<Test>::get(id).unwrap().supply);
    assert_eq!(
      Accounts::<Test>::get(target, id).unwrap().balance,
      acc_balance - amount
    );
  })
}

#[test]
fn decrease_balance_topup_check() {
  new_test_ext().execute_with(|| {
    let id = create_topupped_fa("fa name", 2);
    let target = 3; // account has 20 of fa 1 and topup speed 5
    assert_ok!(FungibleAssets::do_mint(id, &target, 20.into()));

    let amount = 3.into(); // total suply 20
    let max_allowed = false;
    let acc_balance = Accounts::<Test>::get(target, id).unwrap().balance;

    assert!(TopUpQueue::<Test>::get(id, target).is_none());
    assert_eq!(
      FungibleAssets::decrease_balance(id, &target, amount, max_allowed),
      Ok(amount)
    );
    assert_eq!(
      Accounts::<Test>::get(target, id).unwrap().balance,
      acc_balance - amount // 17
    );
    assert!(TopUpQueue::<Test>::contains_key(id, target));

    assert_eq!(
      FungibleAssets::decrease_balance(id, &target, 15.into(), max_allowed),
      Ok(15.into())
    );
    assert!(TopUpQueue::<Test>::contains_key(id, target));

    assert_eq!(
      FungibleAssets::decrease_balance(id, &target, 1000.into(), true),
      Ok(2.into())
    );
    assert!(TopUpQueue::<Test>::contains_key(id, target));
  })
}

#[test]
fn decrease_balance_event() {
  new_test_ext().execute_with(|| {
    let id = create_fa("fa name", 2);

    let target = 1; // account has 1_000 of fa 0
    assert_ok!(FungibleAssets::do_mint(id, &target, 1000.into()));

    System::reset_events();
    let amount = 900.into(); // total suply 11_000
    let max_allowed = true;
    assert_ok!(FungibleAssets::decrease_balance(
      id,
      &target,
      amount,
      max_allowed
    ));
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: FaEvent::Burned {
          asset_id: id,
          owner: target,
          balance: amount
        }
        .into(),
        topics: vec![],
      },]
    );
  })
}

#[test]
fn top_upped_asset_manipulations_test() {
  new_test_ext().execute_with(|| {
    let curr_len = TopUppedAssets::<Test>::get().len();
    assert_ok!(FungibleAssets::top_upped_asset_add(&33.into()));
    assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
    assert!(TopUppedAssets::<Test>::get().contains(&33.into()));

    assert_ok!(FungibleAssets::top_upped_asset_add(&33.into()));
    assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
    assert!(TopUppedAssets::<Test>::get().contains(&33.into()));

    let curr_len = TopUppedAssets::<Test>::get().len();
    assert_ok!(FungibleAssets::top_upped_asset_add(&34.into()));
    assert_eq!(curr_len + 1, TopUppedAssets::<Test>::get().len());
    assert!(TopUppedAssets::<Test>::get().contains(&34.into()));

    let curr_len = TopUppedAssets::<Test>::get().len();
    FungibleAssets::top_upped_asset_remove(&34.into());
    assert_eq!(curr_len - 1, TopUppedAssets::<Test>::get().len());
    assert!(!TopUppedAssets::<Test>::get().contains(&34.into()));

    let curr_len = TopUppedAssets::<Test>::get().len();
    FungibleAssets::top_upped_asset_remove(&34.into());
    assert_eq!(curr_len, TopUppedAssets::<Test>::get().len());
    assert!(!TopUppedAssets::<Test>::get().contains(&34.into()));

    let curr_len = TopUppedAssets::<Test>::get().len();
    FungibleAssets::top_upped_asset_remove(&33.into());
    assert_eq!(curr_len - 1, TopUppedAssets::<Test>::get().len());
    assert!(!TopUppedAssets::<Test>::get().contains(&33.into()));
  })
}

#[test]
fn top_upped_asset_remove_from_queue() {
  new_test_ext().execute_with(|| {
    let id = create_topupped_fa("fa name", 2);

    // in genesis we have one asset with topup
    assert_eq!(TopUppedAssets::<Test>::get().len(), 1);
    // add fake record to TopUpQueue and check removing
    assert!(TopUppedAssets::<Test>::get().contains(&id.into()));
    TopUpQueue::<Test>::insert(&FungibleAssetId::from(id), &3, ());

    assert!(TopUpQueue::<Test>::get(&FungibleAssetId::from(id), &3).is_some());
    TopUpQueue::<Test>::get(&FungibleAssetId::from(id), &3).unwrap();
    FungibleAssets::top_upped_asset_remove(&FungibleAssetId::from(id));
    assert!(TopUpQueue::<Test>::get(&FungibleAssetId::from(id), &3).is_none());
  })
}

#[test]
fn process_top_upped_assets() {
  new_test_ext().execute_with(|| {
    // create several accounts with balances
    let id = create_topupped_fa("fa name", 2);

    assert_ok!(FungibleAssets::increase_balance(id, &1000, 20.into()));
    assert_ok!(FungibleAssets::increase_balance(id, &1100, 17.into()));
    assert_ok!(FungibleAssets::increase_balance(id, &1200, 1.into()));
    // add it to queue
    TopUpQueue::<Test>::insert(&id, &1100, ());
    TopUpQueue::<Test>::insert(&id, &1200, ());

    assert_eq!(FungibleAssets::process_top_upped_assets(), 0);

    assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20.into());
    assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 6.into());
    assert!(!TopUpQueue::<Test>::contains_key(&id, &1100));
    assert!(TopUpQueue::<Test>::contains_key(&id, &1200));
  })
}

#[test]
fn process_top_up_in_progress() {
  new_test_ext().execute_with(|| {
    // create several accounts with balances
    let id = create_topupped_fa("fa name", 2);

    assert_ok!(FungibleAssets::increase_balance(id, &1100, 17.into()));
    assert_ok!(FungibleAssets::increase_balance(id, &1200, 1.into()));
    // add it to queue
    TopUpQueue::<Test>::insert(&id, &1100, ());
    TopUpQueue::<Test>::insert(&id, &1200, ());

    run_to_block(System::block_number() + 1);

    assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20.into());
    assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 6.into());
    assert!(!TopUpQueue::<Test>::contains_key(&id, &1100));
    assert!(TopUpQueue::<Test>::contains_key(&id, &1200));

    run_to_block(System::block_number() + 5);

    assert_eq!(Accounts::<Test>::get(1100, id).unwrap().balance, 20.into());
    assert_eq!(Accounts::<Test>::get(1200, id).unwrap().balance, 20.into());
    assert!(!TopUpQueue::<Test>::contains_key(&id, &1100));
    assert!(!TopUpQueue::<Test>::contains_key(&id, &1200));
  })
}

#[test]
fn inc_dec_references_worked() {
  new_test_ext().execute_with(|| {
    // create several accounts with balances
    let id = create_topupped_fa("fa name", 2);

    assert_ok!(FungibleAssets::inc_references(&id));
    assert_eq!(Assets::<Test>::get(id).unwrap().references, 1);
    assert_ok!(FungibleAssets::inc_references(&id));
    assert_eq!(Assets::<Test>::get(id).unwrap().references, 2);
    assert_ok!(FungibleAssets::dec_references(&id));
    assert_eq!(Assets::<Test>::get(id).unwrap().references, 1);
    assert_ok!(FungibleAssets::dec_references(&id));
    assert_eq!(Assets::<Test>::get(id).unwrap().references, 0);
  })
}
#[test]
fn dec_references_err() {
  new_test_ext().execute_with(|| {
    // create several accounts with balances
    let id = create_topupped_fa("fa name", 2);

    assert_eq!(Assets::<Test>::get(id).unwrap().references, 0);
    assert_noop!(
      FungibleAssets::dec_references(&id),
      ArithmeticError::Underflow
    );
  })
}

#[test]
fn increase_balance_topup_check() {
  new_test_ext().execute_with(|| {
    let id = create_topupped_fa("fa name", 2);

    let beneficiary = 333; // account has no fa 1
    let amount = 3.into(); // total suply 20

    assert!(TopUpQueue::<Test>::get(id, beneficiary).is_none());
    assert!(Accounts::<Test>::get(beneficiary, id).is_none());
    assert_ok!(FungibleAssets::increase_balance(id, &beneficiary, amount));

    assert_eq!(
      Accounts::<Test>::get(beneficiary, id).unwrap().balance,
      amount
    );
    assert!(TopUpQueue::<Test>::get(id, beneficiary).is_none());

    assert_ok!(FungibleAssets::increase_balance(
      id,
      &beneficiary,
      15.into()
    ));
    assert!(TopUpQueue::<Test>::get(id, beneficiary).is_none());

    assert_ok!(FungibleAssets::increase_balance(
      id,
      &beneficiary,
      15.into()
    ));
    assert!(TopUpQueue::<Test>::get(id, beneficiary).is_none());
  })
}
