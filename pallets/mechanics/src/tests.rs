use super::*;
use crate::{mock::*, Error, Something, Config, Timeouts, MechanicId};
use frame_support::{assert_noop, assert_ok};

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
		let id = MechanicId::<Test>::from_account_id(acc);
		assert_eq!(acc, id.account_id);
		assert_eq!(n+1, id.nonce);
	});
}

#[test]
fn init_mechanic_set_timeout() {
	new_test_ext().execute_with(|| {
		let acc = 222;
		System::inc_account_nonce(acc);
		System::set_block_number(2);
		let b = System::block_number();

		let id = MechanicsModule::init_mechanic(acc);
		assert_eq!(Timeouts::<Test>::contains_key(
			(
				b+20,
				id.account_id,
				id.nonce,
			)), true);
	});
}
