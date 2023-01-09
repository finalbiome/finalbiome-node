use crate::{mock::*, Event as TestEvent, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
  new_test_ext(1).execute_with(|| {
    // Dispatch a signed extrinsic.
    assert_ok!(Users::do_something(Origin::signed(1), 42));
    // Read pallet storage and assert an expected result.
    assert_eq!(Users::something(), Some(42));
  });
}

#[test]
fn correct_error_for_none_value() {
  new_test_ext(1).execute_with(|| {
    // Ensure the expected error is thrown when no value is present.
    assert_noop!(
      Users::cause_error(Origin::signed(1)),
      Error::<Test>::NoneValue
    );
  });
}

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
		// A non-registrar `key` will trigger a `RequireRegistrar` error and a non-registrar `key` cannot change
		// the registrar `key`.
		assert_noop!(Users::set_registrar_key(Origin::signed(2), 3), Error::<Test>::RequireRegistrar);
	});
}

#[test]
fn set_registrar_key_emits_events_correctly() {
	new_test_ext(1).execute_with(|| {
		// Set block number to 1 because events are not emitted on block 0.
		System::set_block_number(1);

		// A registrar `key` can change the registrar `key`.
		assert_ok!(Users::set_registrar_key(Origin::signed(1), 2));
		System::assert_has_event(Event::Users(TestEvent::KeyChanged { old_registrar: Some(1) }));
		// Double check.
		assert_ok!(Users::set_registrar_key(Origin::signed(2), 4));
		System::assert_has_event(Event::Users(TestEvent::KeyChanged { old_registrar: Some(2) }));
	});
}
