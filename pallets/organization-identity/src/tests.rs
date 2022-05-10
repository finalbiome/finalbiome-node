use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::ensure_signed;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(OrganizationIdentity::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(OrganizationIdentity::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(OrganizationIdentity::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn create_organization_works() {
	new_test_ext().execute_with(|| {
		// Create org with some name
		let name = br"some name".to_vec();
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(1), name.clone()));
		// Read pallet storage and assert an expected result.
		let org = ensure_signed(Origin::signed(1)).unwrap();
		

		let stored_org = OrganizationIdentity::organizations(org);
		assert_eq!(stored_org.unwrap().name.to_vec(), name);

	})
}

#[test]
fn create_organization_exceed_name_length() {
	new_test_ext().execute_with(|| {
		// Create org with long name
		let name = br"some name012".to_vec();
		assert_noop!(OrganizationIdentity::create_organization(Origin::signed(1), name), Error::<Test>::OrganizationNameTooLong);
	})
}

#[test]
fn create_organization_already_exist() {
	new_test_ext().execute_with(|| {
		// try create an organization second time
		// Create org with some name
		let name = br"some name".to_vec();
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(1), name));
		let name2 = br"some name2".to_vec();
		// Ensure the expected error is thrown when the account is already an organization.
		assert_noop!(OrganizationIdentity::create_organization(Origin::signed(1), name2), Error::<Test>::OrganizationExists);
	})
}
