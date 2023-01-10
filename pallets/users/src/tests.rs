use super::*;
use frame_support::{
  assert_noop, assert_ok,
  traits::{fungible::Mutate, tokens::imbalance::Imbalance, Currency},
};
use mock::{new_test_ext, run_to_block, Balances, Event as TestEvent, Origin, System, Test, Users};

#[test]
fn test_setup_works() {
  // Environment setup, registrar `key` retrieval should work as expected.
  new_test_ext(1).execute_with(|| {
    assert_eq!(Users::registrar_key(), Some(1u64));
  });
}

#[test]
fn set_registrar_key_basics() {
  new_test_ext(1).execute_with(|| {
    // A registrar `key` can change the registrar `key`
    assert_ok!(Users::set_registrar_key(Origin::signed(1), 2));
    assert_eq!(Users::registrar_key(), Some(2u64));
  });

  new_test_ext(1).execute_with(|| {
    // A non-registrar `key` will trigger a `RequireRegistrar` error and a non-registrar `key`
    // cannot change the registrar `key`.
    assert_noop!(
      Users::set_registrar_key(Origin::signed(2), 3),
      Error::<Test>::RequireRegistrar
    );
  });
}

#[test]
fn set_registrar_key_emits_events_correctly() {
  new_test_ext(1).execute_with(|| {
    // Set block number to 1 because events are not emitted on block 0.
    System::set_block_number(1);

    // A registrar `key` can change the registrar `key`.
    assert_ok!(Users::set_registrar_key(Origin::signed(1), 2));
    System::assert_has_event(TestEvent::Users(Event::KeyChanged {
      old_registrar: Some(1),
    }));
    // Double check.
    assert_ok!(Users::set_registrar_key(Origin::signed(2), 4));
    System::assert_has_event(TestEvent::Users(Event::KeyChanged {
      old_registrar: Some(2),
    }));
  });
}

#[test]
fn push_to_slot_basics() {
  new_test_ext(1).execute_with(|| {
    // Account can be added to slot
    let slot = 2;
    let target = 3;

    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 0);

    assert_ok!(Users::push_to_slot(slot, &target));
    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 1);
    assert!(accs.contains(&target));

    let target = 4;
    assert_ok!(Users::push_to_slot(slot, &target));
    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 2);
    assert!(accs.contains(&target));
  });
}

#[test]
fn do_sign_up_basics() {
  new_test_ext(1).execute_with(|| {
    // Checks what the account added to some slot and credit needed amount of tokens

    let target = 3;
    // target has no any tokens
    let acc_data = System::account(target);
    assert_eq!(acc_data.data.free, 0);
    assert_eq!(Balances::total_issuance(), 0);

    System::set_block_number(2); // mocked recovery period = 5;

    assert!(!SlotsLookup::<Test>::contains_key(&target));
    assert_ok!(Users::do_sign_up(target));

    let accs = Slots::<Test>::get(2);
    assert_eq!(accs.len(), 1);
    assert!(accs.contains(&target));
    assert!(SlotsLookup::<Test>::contains_key(&target));

    let acc_data = System::account(target);
    assert_eq!(acc_data.data.free, 100);
    assert_eq!(Balances::total_issuance(), 100);
  });

  new_test_ext(1).execute_with(|| {
    // Can't register twice
    System::set_block_number(2); // mocked recovery period = 5;
    let target = 3;
    assert_ok!(Users::do_sign_up(target));

    assert_noop!(Users::do_sign_up(target), Error::<Test>::Registered);
  });
}

#[test]
fn restore_quota() {
  new_test_ext(1).execute_with(|| {
    // Checks the restoration of the quota if not enought
    let target = 3;
    // set start position
    let _ = Balances::deposit_creating(&target, 50);
    let acc_data = System::account(target);
    assert_eq!(acc_data.data.free, 50);
    assert_eq!(Balances::total_issuance(), 50);
    // restore quota
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 50);
    // restore for empty account
    let target = 4;
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 100);
  });

  new_test_ext(1).execute_with(|| {
    // Checks the restoration of the quota if there are enough tokens
    let target = 3;
    // set start position
    let _ = Balances::deposit_creating(&target, 100);
    // try to restore quota
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 0);
    // restore for empty account
    let target = 4;
    let _ = Balances::deposit_creating(&target, 300);
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 0);
  });
}

#[test]
fn service_quotas_empty() {
  new_test_ext(1).execute_with(|| {
    // Checks with empty slot
    assert_eq!(Balances::total_issuance(), 0);
    Users::service_quotas(3);

    assert_eq!(Balances::total_issuance(), 0);
  });

  new_test_ext(1).execute_with(|| {
    // Checks with slot where accounts are full
    let slot = 4;
    let _ = Balances::deposit_creating(&3, 100);
    let _ = Balances::deposit_creating(&4, 200);
    assert_eq!(Balances::free_balance(&3), 100);
    assert_eq!(Balances::free_balance(&4), 200);
    assert_eq!(Balances::total_issuance(), 300);

    assert_ok!(Users::push_to_slot(slot, &3));
    assert_ok!(Users::push_to_slot(slot, &4));

    Users::service_quotas(slot);

    assert_eq!(Balances::free_balance(&3), 100);
    assert_eq!(Balances::free_balance(&4), 200);

    assert_eq!(Balances::total_issuance(), 300);
  });
}

#[test]
fn service_quotas_basics() {
  new_test_ext(1).execute_with(|| {
    // Checks with four accounts where two is full
    let slot = 4;
    let _ = Balances::deposit_creating(&3, 50);
    let _ = Balances::deposit_creating(&4, 200);
    let _ = Balances::deposit_creating(&5, 30);
    let _ = Balances::deposit_creating(&6, 100);
    assert_eq!(Balances::free_balance(&3), 50);
    assert_eq!(Balances::free_balance(&4), 200);
    assert_eq!(Balances::free_balance(&5), 30);
    assert_eq!(Balances::free_balance(&6), 100);
    assert_eq!(Balances::total_issuance(), 380);

    assert_ok!(Users::push_to_slot(slot, &3));
    assert_ok!(Users::push_to_slot(slot, &4));
    assert_ok!(Users::push_to_slot(slot, &5));
    assert_ok!(Users::push_to_slot(slot, &6));

    Users::service_quotas(slot);

    assert_eq!(Balances::free_balance(&3), 100);
    assert_eq!(Balances::free_balance(&4), 200);
    assert_eq!(Balances::free_balance(&5), 100);
    assert_eq!(Balances::free_balance(&6), 100);
    assert_eq!(Balances::total_issuance(), 500);
  });
}

#[test]
fn service_quotas_each_block() {
  new_test_ext(1).execute_with(|| {
    let slot = 3;
    let target = 3;
    assert_eq!(Balances::total_issuance(), 0);

    System::set_block_number(1);

    run_to_block(System::block_number() + 2);
    assert_eq!(Balances::total_issuance(), 0);

    assert!(!Slots::<Test>::contains_key(slot));

    assert_ok!(Users::do_sign_up(target));

    assert!(Slots::<Test>::contains_key(slot));
    assert_eq!(Balances::free_balance(&target), 100);
    assert_eq!(Balances::total_issuance(), 100);

    run_to_block(System::block_number() + 1);

    let _ = Balances::burn_from(&target, 15);
    assert_eq!(Balances::free_balance(&target), 100 - 15);
    assert_eq!(Balances::total_issuance(), 100 - 15);

    run_to_block(System::block_number() + 3);
    assert_eq!(Balances::free_balance(&target), 100 - 15);
    assert_eq!(Balances::total_issuance(), 100 - 15);

    run_to_block(System::block_number() + 1);
    assert_eq!(Balances::free_balance(&target), 100);
    assert_eq!(Balances::total_issuance(), 100);
  });
}
