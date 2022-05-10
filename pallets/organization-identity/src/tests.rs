use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::{ensure_signed};

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

		// TODO: test the events.
		//			 Impl bellow doesn't work
		// System::assert_has_event(Event::OrganizationIdentity(crate::Event::AddedToOrganization(name, org)));

		assert_noop!(OrganizationIdentity::create_organization(Origin::none(), name.clone()), sp_runtime::traits::BadOrigin);
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

#[test]
fn add_member_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(OrganizationIdentity::add_member(Origin::none(), 2), sp_runtime::traits::BadOrigin);

		// Create org 
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(1), br"some name".to_vec()));
		// add member with id 2
		assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));
		assert_eq!(OrganizationIdentity::member_count(1), 1u8);
		assert_eq!(OrganizationIdentity::member_count(0), 0u8);
		// add member with id 3
		assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 3));
		assert_eq!(OrganizationIdentity::member_count(1), 2u8);
		// add member with id 2 second time
		assert_noop!(OrganizationIdentity::add_member(Origin::signed(1), 2), Error::<Test>::AlreadyMember);
		assert_eq!(OrganizationIdentity::member_count(1), 2u8);
		// add member with id 4
		assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 4));
		assert_eq!(OrganizationIdentity::member_count(1), 3u8);
		// add member with id 5 should return an error
		assert_noop!(OrganizationIdentity::add_member(Origin::signed(1), 5), Error::<Test>::MembershipLimitReached);

		// add member to not org
		assert_noop!(OrganizationIdentity::add_member(Origin::signed(2), 5), Error::<Test>::NotOrganization);


		// Ensure the expected error is thrown when the account is already an organization.
		// assert_noop!(OrganizationIdentity::create_organization(Origin::signed(1), name2), Error::<Test>::OrganizationExists);
	})
}

#[test]
fn add_member_not_org() {
	new_test_ext().execute_with(|| {
		// Organization can't be a member
		// Create org 1
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(1), br"some name".to_vec()));
		// Create org 2
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(2), br"some name".to_vec()));
		// add org 1 as member to org 2
		assert_noop!(OrganizationIdentity::add_member(Origin::signed(2), 1), Error::<Test>::InvalidMember);

	})
}

#[test]
fn remove_member_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(OrganizationIdentity::add_member(Origin::none(), 2), sp_runtime::traits::BadOrigin);

		// Create org id 1
		assert_ok!(OrganizationIdentity::create_organization(Origin::signed(1), br"some name".to_vec()));
		// add member with id 2
		assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));
		assert_eq!(OrganizationIdentity::member_count(1), 1u8);
		assert_eq!(OrganizationIdentity::member_count(0), 0u8);
		// add member with id 3
		assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 3));
		assert_eq!(OrganizationIdentity::member_count(1), 2u8);
		// remove member 2
		assert_ok!(OrganizationIdentity::remove_member(Origin::signed(1), 2));
		assert_eq!(OrganizationIdentity::member_count(1), 1u8);
		// remove member 4
		assert_noop!(OrganizationIdentity::remove_member(Origin::signed(1), 4), Error::<Test>::NotMember);
		assert_eq!(OrganizationIdentity::member_count(1), 1u8);
		// remove member as not org
		assert_noop!(OrganizationIdentity::remove_member(Origin::signed(2), 4), Error::<Test>::NotOrganization);
		assert_eq!(OrganizationIdentity::member_count(1), 1u8);
	})
}
