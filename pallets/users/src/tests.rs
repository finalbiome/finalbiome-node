use super::*;
use frame_support::{
  assert_noop, assert_ok,
  traits::{fungible::Mutate, tokens::imbalance::Imbalance, Currency},
};
use mock::{new_test_ext, run_to_block, Balances, Event as TestEvent, Origin, System, Test, Users};
use sp_core::Pair;
use sp_runtime::AccountId32;

use hex_literal::hex;

fn generate_account() -> AccountId32 {
  let (pair, _) = sp_core::sr25519::Pair::generate();
  pair.public().into()
}

#[test]
fn test_setup_works() {
  let registrar = generate_account();
  // Environment setup, registrar `key` retrieval should work as expected.
  new_test_ext(&registrar).execute_with(|| {
    assert_eq!(Users::registrar_key(), Some(registrar));
  });
}

#[test]
fn set_registrar_key_basics() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // A registrar `key` can change the registrar `key`
    let registrar_new = generate_account();
    assert_ok!(Users::set_registrar_key(
      Origin::signed(registrar.clone()),
      registrar_new.clone()
    ));
    assert_eq!(Users::registrar_key(), Some(registrar_new));
  });

  new_test_ext(&registrar).execute_with(|| {
    // A non-registrar `key` will trigger a `RequireRegistrar` error and a non-registrar `key`
    // cannot change the registrar `key`.
    let registrar_new = generate_account();
    let registrar_fake = generate_account();

    assert_noop!(
      Users::set_registrar_key(Origin::signed(registrar_new), registrar_fake),
      Error::<Test>::RequireRegistrar
    );
  });
}

#[test]
fn set_registrar_key_emits_events_correctly() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Set block number to 1 because events are not emitted on block 0.
    System::set_block_number(1);

    // A registrar `key` can change the registrar `key`.
    let registrar_new = generate_account();

    assert_ok!(Users::set_registrar_key(
      Origin::signed(registrar.clone()),
      registrar_new.clone()
    ));
    System::assert_has_event(TestEvent::Users(Event::KeyChanged {
      old_registrar: Some(registrar.clone()),
    }));
    // Double check.
    let registrar_new2 = generate_account();
    assert_ok!(Users::set_registrar_key(
      Origin::signed(registrar_new.clone()),
      registrar_new2
    ));
    System::assert_has_event(TestEvent::Users(Event::KeyChanged {
      old_registrar: Some(registrar_new),
    }));
  });
}

#[test]
fn push_to_slot_basics() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Account can be added to slot
    let slot = 2;
    let target = generate_account();

    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 0);

    assert_ok!(Users::push_to_slot(slot, &target));
    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 1);
    assert!(accs.contains(&target));

    let target = generate_account();
    assert_ok!(Users::push_to_slot(slot, &target));
    let accs = Slots::<Test>::get(slot);
    assert_eq!(accs.len(), 2);
    assert!(accs.contains(&target));
  });
}

#[test]
fn do_sign_up_basics() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Checks what the account added to some slot and credit needed amount of tokens

    let target = generate_account();
    // target has no any tokens
    let acc_data = System::account(target.clone());
    assert_eq!(acc_data.data.free, 0);
    assert_eq!(Balances::total_issuance(), 0);

    System::set_block_number(2); // mocked recovery period = 5;

    assert!(!SlotsLookup::<Test>::contains_key(&target));
    assert_ok!(Users::do_sign_up(target.clone()));

    let accs = Slots::<Test>::get(2);
    assert_eq!(accs.len(), 1);
    assert!(accs.contains(&target));
    assert!(SlotsLookup::<Test>::contains_key(&target));

    let acc_data = System::account(target);
    assert_eq!(acc_data.data.free, 100);
    assert_eq!(Balances::total_issuance(), 100);
  });

  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Can't register twice
    System::set_block_number(2); // mocked recovery period = 5;
    let target = generate_account();
    assert_ok!(Users::do_sign_up(target.clone()));

    assert_noop!(Users::do_sign_up(target), Error::<Test>::Registered);
  });
}

#[test]
fn restore_quota() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Checks the restoration of the quota if not enought
    let target = generate_account();
    // set start position
    let _ = Balances::deposit_creating(&target, 50);
    let acc_data = System::account(target.clone());
    assert_eq!(acc_data.data.free, 50);
    assert_eq!(Balances::total_issuance(), 50);
    // restore quota
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 50);
    // restore for empty account
    let target = generate_account();
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 100);
  });

  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Checks the restoration of the quota if there are enough tokens
    let target = generate_account();
    // set start position
    let _ = Balances::deposit_creating(&target, 100);
    // try to restore quota
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 0);
    // restore for empty account
    let target = generate_account();
    let _ = Balances::deposit_creating(&target, 300);
    let imb = Users::restore_quota(&target, 100);
    assert_eq!(imb.peek(), 0);
  });
}

#[test]
fn service_quotas_empty() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Checks with empty slot
    assert_eq!(Balances::total_issuance(), 0);
    Users::service_quotas(3);

    assert_eq!(Balances::total_issuance(), 0);
  });

  new_test_ext(&registrar).execute_with(|| {
    // Checks with slot where accounts are full
    let slot = 4;
    let target_1 = generate_account();
    let target_2 = generate_account();
    let _ = Balances::deposit_creating(&target_1, 100);
    let _ = Balances::deposit_creating(&target_2, 200);
    assert_eq!(Balances::free_balance(&target_1), 100);
    assert_eq!(Balances::free_balance(&target_2), 200);
    assert_eq!(Balances::total_issuance(), 300);

    assert_ok!(Users::push_to_slot(slot, &target_1));
    assert_ok!(Users::push_to_slot(slot, &target_2));

    Users::service_quotas(slot);

    assert_eq!(Balances::free_balance(&target_1), 100);
    assert_eq!(Balances::free_balance(&target_2), 200);

    assert_eq!(Balances::total_issuance(), 300);
  });
}

#[test]
fn service_quotas_basics() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    // Checks with four accounts where two is full
    let slot = 4;
    let target_1 = generate_account();
    let target_2 = generate_account();
    let target_3 = generate_account();
    let target_4 = generate_account();
    let _ = Balances::deposit_creating(&target_1, 50);
    let _ = Balances::deposit_creating(&target_2, 200);
    let _ = Balances::deposit_creating(&target_3, 30);
    let _ = Balances::deposit_creating(&target_4, 100);
    assert_eq!(Balances::free_balance(&target_1), 50);
    assert_eq!(Balances::free_balance(&target_2), 200);
    assert_eq!(Balances::free_balance(&target_3), 30);
    assert_eq!(Balances::free_balance(&target_4), 100);
    assert_eq!(Balances::total_issuance(), 380);

    assert_ok!(Users::push_to_slot(slot, &target_1));
    assert_ok!(Users::push_to_slot(slot, &target_2));
    assert_ok!(Users::push_to_slot(slot, &target_3));
    assert_ok!(Users::push_to_slot(slot, &target_4));

    Users::service_quotas(slot);

    assert_eq!(Balances::free_balance(&target_1), 100);
    assert_eq!(Balances::free_balance(&target_2), 200);
    assert_eq!(Balances::free_balance(&target_3), 100);
    assert_eq!(Balances::free_balance(&target_4), 100);
    assert_eq!(Balances::total_issuance(), 500);
  });
}

#[test]
fn service_quotas_each_block() {
  let registrar = generate_account();
  new_test_ext(&registrar).execute_with(|| {
    let slot = 3;
    let target = generate_account();
    assert_eq!(Balances::total_issuance(), 0);

    System::set_block_number(1);

    run_to_block(System::block_number() + 2);
    assert_eq!(Balances::total_issuance(), 0);

    assert!(!Slots::<Test>::contains_key(slot));

    assert_ok!(Users::do_sign_up(target.clone()));

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

#[test]
fn verify_signature_works() {
  let registrar = sp_core::sr25519::Public::from_raw(hex!(
    "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
  ))
  .into();

  new_test_ext(&registrar).execute_with(|| {

    // seed from response
    let user = sp_core::sr25519::Pair::from_seed(&hex!(
        "ea2ea5ebebc4f1216acdf1a0fd56d694ad827c952cc9578dde1f35c2a50a1fcf"
      ))
      .public()
      .into();
    // sign from response
    let sig = sp_core::sr25519::Signature::from_raw(hex!(
        "9c570151138cf8c08a9ed3b2d9ff330e05ee39344e4efb488adb9f285b51751bb137042b188f235b32dd7786824a996b9096260dded8a24ee1bddd53978f7683"
      ));

    let signature =
      <sp_core::sr25519::Signature as AsRef<[u8; 64]>>::as_ref(&sig)
      .to_vec()
      .try_into()
      .unwrap();

    assert_ok!(Users::verify_signature(&user, signature));
  })
}

#[test]
fn verify_signature_works_invalid_sign() {
  // signs by other than registerer account
  let registrar = sp_core::sr25519::Public::from_raw(hex!(
    "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
  ))
  .into();

  new_test_ext(&registrar).execute_with(|| {

    // seed from response
    let user = sp_core::sr25519::Pair::from_seed(&hex!(
        "ea2ea5ebebc4f1216acdf1a0fd56d694ad827c952cc9578dde1f35c2a50a1fcf"
      ))
      .public()
      .into();
    // sign from response
    let sig = sp_core::sr25519::Signature::from_raw(hex!(
        "9c570151138cf8c08a9ed3b2d9ff330e05ee39344e4efb488adb9f285b51751bb137042b188f235b32dd7786824a996b9096260dded8a24ee1bddd53978f7683"
      ));

    let signature =
      <sp_core::sr25519::Signature as AsRef<[u8; 64]>>::as_ref(&sig)
      .to_vec()
      .try_into()
      .unwrap();

    assert_noop!(Users::verify_signature(&user, signature), Error::<Test>::InvalidSignature);
  })
}
