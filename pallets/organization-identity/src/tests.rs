use crate::{
  mock::*, AirDropAsset, Error, Event as OrgEvent, Members, OnboardingAssets, Organizations,
  UsersOf,
};
use frame_support::{assert_noop, assert_ok};
use frame_system::{ensure_signed, EventRecord, Phase};
use pallet_support::Attribute;
use sp_runtime::DispatchError;

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
fn create_organization_works() {
  new_test_ext().execute_with(|| {
    // Create org with some name
    let name = br"some name".to_vec();
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      name.clone()
    ));
    // Read pallet storage and assert an expected result.
    let org = ensure_signed(Origin::signed(1)).unwrap();

    let stored_org = Organizations::<Test>::get(&org).unwrap();
    assert_eq!(stored_org.name.to_vec(), name);

    assert_eq!(Members::<Test>::contains_key(&org), true);

    // TODO: test the events.
    //			 Impl bellow doesn't work
    // System::assert_has_event(Event::OrganizationIdentity(crate::Event::AddedToOrganization(name,
    // org)));

    assert_noop!(
      OrganizationIdentity::create_organization(Origin::none(), name),
      sp_runtime::traits::BadOrigin
    );
  })
}

#[test]
fn create_organization_exceed_name_length() {
  new_test_ext().execute_with(|| {
    // Create org with long name
    let name = br"some name012".to_vec();
    assert_noop!(
      OrganizationIdentity::create_organization(Origin::signed(1), name),
      Error::<Test>::OrganizationNameTooLong
    );
  })
}

#[test]
fn create_organization_already_exist() {
  new_test_ext().execute_with(|| {
    // try create an organization second time
    // Create org with some name
    let name = br"some name".to_vec();
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      name
    ));
    let name2 = br"some name2".to_vec();
    // Ensure the expected error is thrown when the account is already an organization.
    assert_noop!(
      OrganizationIdentity::create_organization(Origin::signed(1), name2),
      Error::<Test>::OrganizationExists
    );
  })
}

#[test]
fn add_member_works() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      OrganizationIdentity::add_member(Origin::none(), 2),
      sp_runtime::traits::BadOrigin
    );

    // Create org
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    // add member with id 2
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));
    assert_eq!(Members::<Test>::contains_key(&2), true);
    assert_eq!(OrganizationIdentity::member_count(1), 1u8);
    assert_eq!(OrganizationIdentity::member_count(0), 0u8);
    // add member with id 3
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 3));
    assert_eq!(Members::<Test>::contains_key(&3), true);
    assert_eq!(OrganizationIdentity::member_count(1), 2u8);
    // add member with id 2 second time
    assert_noop!(
      OrganizationIdentity::add_member(Origin::signed(1), 2),
      Error::<Test>::AlreadyMember
    );
    assert_eq!(OrganizationIdentity::member_count(1), 2u8);
    // add member with id 4
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 4));
    assert_eq!(Members::<Test>::contains_key(&4), true);
    assert_eq!(OrganizationIdentity::member_count(1), 3u8);
    // add member with id 5 should return an error
    assert_noop!(
      OrganizationIdentity::add_member(Origin::signed(1), 5),
      Error::<Test>::MembershipLimitReached
    );

    // add member to not org
    assert_noop!(
      OrganizationIdentity::add_member(Origin::signed(2), 5),
      Error::<Test>::NotOrganization
    );

    // Ensure the expected error is thrown when the account is already an organization.
    // assert_noop!(OrganizationIdentity::create_organization(Origin::signed(1), name2),
    // Error::<Test>::OrganizationExists);
  })
}

#[test]
fn add_member_not_org() {
  new_test_ext().execute_with(|| {
    // Organization can't be a member
    // Create org 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    // Create org 2
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(2),
      br"some name".to_vec()
    ));
    // add org 1 as member to org 2
    assert_noop!(
      OrganizationIdentity::add_member(Origin::signed(2), 1),
      Error::<Test>::InvalidMember
    );
  })
}

#[test]
fn remove_member_works() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      OrganizationIdentity::add_member(Origin::none(), 2),
      sp_runtime::traits::BadOrigin
    );

    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    // add member with id 2
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));
    assert_eq!(OrganizationIdentity::member_count(1), 1u8);
    assert_eq!(OrganizationIdentity::member_count(0), 0u8);
    // add member with id 3
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 3));
    assert_eq!(OrganizationIdentity::member_count(1), 2u8);
    // remove member 2
    assert_ok!(OrganizationIdentity::remove_member(Origin::signed(1), 2));
    assert_eq!(Members::<Test>::contains_key(&2), false);
    assert_eq!(OrganizationIdentity::member_count(1), 1u8);
    // remove member 4
    assert_noop!(
      OrganizationIdentity::remove_member(Origin::signed(1), 4),
      Error::<Test>::NotMember
    );
    assert_eq!(OrganizationIdentity::member_count(1), 1u8);
    // remove member as not org
    assert_noop!(
      OrganizationIdentity::remove_member(Origin::signed(2), 4),
      Error::<Test>::NotOrganization
    );
    assert_eq!(OrganizationIdentity::member_count(1), 1u8);
  })
}

#[test]
fn do_set_onboarding_assets_works() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      OrganizationIdentity::do_set_onboarding_assets(&1, None),
      Error::<Test>::NotOrganization
    );

    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    assert_eq!(Organizations::<Test>::contains_key(&1), true);
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      None
    );

    // add assets
    let assets: OnboardingAssets = Some(bvec![
      AirDropAsset::Fa(1.into(), 100.into()),
      AirDropAsset::Nfa(
        1.into(),
        bvec![
          Attribute {
            key: bvec!(br"a"),
            value: 10.try_into().unwrap(),
          },
          Attribute {
            key: bvec!(br"b"),
            value: "t".try_into().unwrap(),
          },
        ]
      )
    ]);
    assert_ok!(OrganizationIdentity::do_set_onboarding_assets(
      &1,
      assets.clone()
    ));
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      assets
    );

    assert_ok!(OrganizationIdentity::do_set_onboarding_assets(&1, None));
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      None
    );
  })
}

#[test]
fn set_onboarding_assets_works() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      OrganizationIdentity::set_onboarding_assets(Origin::none(), 1, None),
      sp_runtime::traits::BadOrigin
    );

    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    // only member can update an organization
    assert_noop!(
      OrganizationIdentity::set_onboarding_assets(Origin::signed(2), 1, None),
      Error::<Test>::NotMember
    );

    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));
    assert_ok!(OrganizationIdentity::set_onboarding_assets(
      Origin::signed(2),
      1,
      None
    ));

    // add assets
    let assets: OnboardingAssets = Some(bvec![
      AirDropAsset::Fa(1.into(), 100.into()),
      AirDropAsset::Nfa(
        1.into(),
        bvec![
          Attribute {
            key: bvec!(br"a"),
            value: 10.try_into().unwrap(),
          },
          Attribute {
            key: bvec!(br"b"),
            value: "t".try_into().unwrap(),
          },
        ]
      )
    ]);
    System::reset_events();
    assert_ok!(OrganizationIdentity::set_onboarding_assets(
      Origin::signed(2),
      1,
      assets
    ));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: OrgEvent::UpdatedOrganization(1).into(),
        topics: vec![],
      },]
    );
  })
}

#[test]
fn do_airdrop_fa_works() {
  new_test_ext().execute_with(|| {
    assert_ok!(OrganizationIdentity::do_airdrop_fa(
      &10,
      10.into(),
      100.into()
    ));
    assert_noop!(
      OrganizationIdentity::do_airdrop_fa(&10, 11.into(), 100.into()),
      DispatchError::Other("mock_do_airdrop_fa_works")
    );
  })
}

#[test]
fn do_airdrop_nfa_works() {
  new_test_ext().execute_with(|| {
    assert_ok!(OrganizationIdentity::do_airdrop_nfa(
      &10,
      10.into(),
      bvec![]
    ));
    assert_noop!(
      OrganizationIdentity::do_airdrop_nfa(&10, 11.into(), bvec![]),
      DispatchError::Other("mock_do_airdrop_nfa_works")
    );
    assert_noop!(
      OrganizationIdentity::do_airdrop_nfa(&10, 12.into(), bvec![]),
      DispatchError::Other("mock_do_airdrop_nfa_works_2")
    );
  })
}

#[test]
fn do_onboarding_works() {
  new_test_ext().execute_with(|| {
    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    assert_eq!(Organizations::<Test>::contains_key(&1), true);
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      None
    );

    // add assets
    let assets: OnboardingAssets = Some(bvec![
      AirDropAsset::Fa(12.into(), 100.into()),
      AirDropAsset::Nfa(
        13.into(),
        bvec![
          Attribute {
            key: bvec!(br"a"),
            value: 10.try_into().unwrap(),
          },
          Attribute {
            key: bvec!(br"b"),
            value: "t".try_into().unwrap(),
          },
        ]
      )
    ]);

    assert_ok!(OrganizationIdentity::do_set_onboarding_assets(
      &1,
      assets.clone()
    ));
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      assets
    );
    assert_eq!(UsersOf::<Test>::contains_key(&1, &333), false);

    assert_ok!(OrganizationIdentity::do_onboarding(&1, &333));
    assert_eq!(UsersOf::<Test>::contains_key(&1, &333), true);
  })
}

#[test]
fn onboarding_works() {
  new_test_ext().execute_with(|| {
    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    assert_eq!(Organizations::<Test>::contains_key(&1), true);
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      None
    );
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));

    // add assets
    let assets: OnboardingAssets = Some(bvec![
      AirDropAsset::Fa(12.into(), 100.into()),
      AirDropAsset::Nfa(
        13.into(),
        bvec![
          Attribute {
            key: bvec!(br"a"),
            value: 10.try_into().unwrap(),
          },
          Attribute {
            key: bvec!(br"b"),
            value: "t".try_into().unwrap(),
          },
        ]
      )
    ]);

    assert_ok!(OrganizationIdentity::do_set_onboarding_assets(
      &1,
      assets.clone()
    ));
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      assets
    );

    assert_noop!(
      OrganizationIdentity::onboarding(Origin::none(), 333),
      sp_runtime::traits::BadOrigin
    );
    assert_noop!(
      OrganizationIdentity::onboarding(Origin::signed(2), 333),
      sp_runtime::traits::BadOrigin
    );

    System::reset_events();

    assert_ok!(OrganizationIdentity::onboarding(Origin::signed(333), 1));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: OrgEvent::Onboard(1, 333).into(),
        topics: vec![],
      },]
    );

    assert_noop!(
      OrganizationIdentity::onboarding(Origin::signed(333), 1),
      Error::<Test>::AlreadyOnboarded
    );
  })
}

#[test]
fn onboarding_works_with_empty_airdrops() {
  new_test_ext().execute_with(|| {
    // Create org id 1
    assert_ok!(OrganizationIdentity::create_organization(
      Origin::signed(1),
      br"some name".to_vec()
    ));
    assert_eq!(Organizations::<Test>::contains_key(&1), true);
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      None
    );
    assert_ok!(OrganizationIdentity::add_member(Origin::signed(1), 2));

    // add assets
    let assets: OnboardingAssets = None;

    assert_ok!(OrganizationIdentity::do_set_onboarding_assets(
      &1,
      assets.clone()
    ));
    assert_eq!(
      Organizations::<Test>::get(&1).unwrap().onboarding_assets,
      assets
    );

    assert_noop!(
      OrganizationIdentity::onboarding(Origin::none(), 333),
      sp_runtime::traits::BadOrigin
    );
    assert_noop!(
      OrganizationIdentity::onboarding(Origin::signed(2), 333),
      sp_runtime::traits::BadOrigin
    );

    System::reset_events();

    assert_ok!(OrganizationIdentity::onboarding(Origin::signed(333), 1));

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: OrgEvent::Onboard(1, 333).into(),
        topics: vec![],
      },]
    );

    assert_noop!(
      OrganizationIdentity::onboarding(Origin::signed(333), 1),
      Error::<Test>::AlreadyOnboarded
    );
  })
}
