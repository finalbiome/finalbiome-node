use super::*;

use crate::{mock::*, ClassDetailsBuilder, Error, Event as NfaEvent};
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};
use pallet_support::{GamerAccount, Locker, MechanicId};

fn get_next_class_id() -> NonFungibleClassId {
  NextClassId::<Test>::get()
}

fn get_next_asset_id() -> NonFungibleAssetId {
  NextAssetId::<Test>::get()
}

#[test]
fn template_test() {
  new_test_ext().execute_with(|| {});
}

#[test]
fn next_class_id_works() {
  new_test_ext().execute_with(|| {
    assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 0.into());
    assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 1.into());
    assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 2.into());
    assert_eq!(NextClassId::<Test>::get(), 3.into());
  });
}

#[test]
fn next_asset_id_works() {
  new_test_ext().execute_with(|| {
    assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 0.into());
    assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 1.into());
    assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 2.into());
    assert_eq!(NextAssetId::<Test>::get(), 3.into());
  });
}

#[test]
fn class_details_builder() {
  new_test_ext().execute_with(|| {
    let b = ClassDetailsBuilder::<Test>::new(1, br"n2345678".to_vec()).unwrap(); // max 8 symbols
    let d = b.build().unwrap();
    assert_eq!(d.name.to_vec(), br"n2345678".to_vec());
    assert_eq!(d.owner, 1);
    assert_eq!(d.instances, 0);
    assert_eq!(d.bettor, None);
  });
}

#[test]
fn class_details_builder_name_len_exceed() {
  new_test_ext().execute_with(|| {
    let name = (0..100)
      .map(|_| "X")
      .collect::<String>()
      .as_bytes()
      .to_vec();
    assert_noop!(
      ClassDetailsBuilder::<Test>::new(1, name),
      Error::<Test>::ClassNameTooLong
    ); // max 64 symbols
  });
}

#[test]
fn asset_details_builder() {
  new_test_ext().execute_with(|| {
    let b = AssetDetailsBuilder::<Test>::new(1).unwrap();
    let d = b.build().unwrap();
    assert_eq!(d.owner, 1);
    assert_eq!(d.locked, Locker::None);
  });
}

#[test]
fn create_class_unsigned() {
  new_test_ext().execute_with(|| {
    let name = br"nfa name".to_vec();
    assert_noop!(
      NonFungibleAssets::create(Origin::none(), 2, name),
      sp_runtime::traits::BadOrigin
    );
  });
}
#[test]
fn create_class_created() {
  new_test_ext().execute_with(|| {
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(
      Origin::signed(1),
      org,
      name.clone()
    ));
    let nfa = Classes::<Test>::get(nfa_id).unwrap();
    assert_eq!(nfa.name.to_vec(), name);
    assert_eq!(nfa.instances, 0);
    assert_eq!(nfa.attributes, 0);
    assert_eq!(nfa.owner, org);
    assert!(ClassAccounts::<Test>::contains_key(org, nfa_id));
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::Created {
          class_id: nfa_id,
          owner: org
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_destroy_class_unknown_class() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      NonFungibleAssets::do_destroy_class(888.into(), Some(999)),
      Error::<Test>::UnknownClass
    );
  });
}

#[test]
fn do_destroy_class_not_owner() {
  new_test_ext().execute_with(|| {
    // create test asset
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    assert_noop!(
      NonFungibleAssets::do_destroy_class(nfa_id, Some(3)),
      Error::<Test>::NoPermission
    );
    assert!(Classes::<Test>::contains_key(nfa_id));
    assert!(ClassAccounts::<Test>::contains_key(org, nfa_id));
  });
}

#[test]
fn do_destroy_class_worked() {
  new_test_ext().execute_with(|| {
    // create test asset
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    System::reset_events();
    assert_ok!(NonFungibleAssets::do_destroy_class(nfa_id, Some(org)));
    assert!(!Classes::<Test>::contains_key(nfa_id));
    assert!(!ClassAccounts::<Test>::contains_key(org, nfa_id));
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::Destroyed { class_id: nfa_id }.into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_destroy_class_removes_attributes() {
  new_test_ext().execute_with(|| {
    // create test class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create attribute
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    assert_ok!(NonFungibleAssets::do_create_attribute(
      class_id,
      Some(org),
      a
    ));
    let key: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    assert!(ClassAttributes::<Test>::contains_key(&class_id, &key));
    assert_eq!(Classes::<Test>::get(&class_id).unwrap().attributes, 1);

    assert_ok!(NonFungibleAssets::do_destroy_class(class_id, Some(org)));
    assert!(!Classes::<Test>::contains_key(&class_id));
    assert!(!ClassAccounts::<Test>::contains_key(&org, &class_id));
    assert!(!ClassAttributes::<Test>::contains_key(&class_id, &key));
  });
}

#[test]
fn destroy_class_not_org() {
  new_test_ext().execute_with(|| {
    // create test asset
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    System::reset_events();
    assert_noop!(
      NonFungibleAssets::destroy(Origin::none(), org, nfa_id),
      sp_runtime::traits::BadOrigin
    );
  });
}

#[test]
fn do_mint_class_unknown_class() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      NonFungibleAssets::do_mint(888.into(), 999),
      Error::<Test>::UnknownClass
    );
  });
}

#[test]
fn do_mint_worked() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    System::reset_events();
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);
    assert!(Assets::<Test>::contains_key(&nfa_id, &id));
    assert!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 1);

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.owner, acc);

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::Issued {
          class_id: nfa_id,
          asset_id: id,
          owner: acc
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_burn_no_attributes() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);
    assert!(Assets::<Test>::contains_key(&nfa_id, &id));
    assert!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 1);

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.owner, acc);

    System::reset_events();
    assert_ok!(NonFungibleAssets::do_burn(nfa_id, id, Some(&acc)));
    assert!(!Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 0);

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::Burned {
          class_id: nfa_id,
          asset_id: id,
          owner: acc
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_burn_with_attributes() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);
    assert!(Assets::<Test>::contains_key(&nfa_id, &id));
    assert!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 1);

    let a = Attribute {
      key: br"a1".to_vec().try_into().unwrap(),
      value: AttributeValue::Number(NumberAttribute {
        number_max: None,
        number_value: 1,
      }),
    };
    let attributes: AttributeList = vec![a.clone()].try_into().unwrap();
    assert_ok!(NonFungibleAssets::assign_attributes(&id, attributes));
    assert!(Attributes::<Test>::contains_key(&id, &a.key));

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.owner, acc);

    System::reset_events();
    assert_ok!(NonFungibleAssets::do_burn(nfa_id, id, Some(&acc)));
    assert!(!Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert!(!Attributes::<Test>::contains_key(&id, a.key));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 0);

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::Burned {
          class_id: nfa_id,
          asset_id: id,
          owner: acc
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_burn_not_owner() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);
    assert!(Assets::<Test>::contains_key(&nfa_id, &id));
    assert!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.owner, acc);

    System::reset_events();
    assert_noop!(
      NonFungibleAssets::do_burn(nfa_id, id, Some(&33)),
      Error::<Test>::NoPermission
    );
    assert!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 1);
  });
}

#[test]
fn do_burn_not_exists() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      NonFungibleAssets::do_burn(11.into(), 22.into(), Some(&33)),
      Error::<Test>::UnknownAsset
    );
  });
}

#[test]
fn do_create_attribute_wrong_attr() {
  new_test_ext().execute_with(|| {
    let nv = NumberAttribute {
      number_value: 100,
      number_max: Some(10),
    };
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: AttributeValue::Number(nv),
    };
    assert_noop!(
      NonFungibleAssets::do_create_attribute(1.into(), None, a),
      DispatchError::Other("Attribute numeric value exceeds the maximum value")
    );
  });
}

#[test]
fn do_create_attribute_class_not_exists() {
  new_test_ext().execute_with(|| {
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    assert_noop!(
      NonFungibleAssets::do_create_attribute(1000.into(), None, a),
      Error::<Test>::UnknownClass
    );
  });
}

#[test]
fn do_create_attribute_owner_no_permissions() {
  new_test_ext().execute_with(|| {
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    // create class
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    assert_noop!(
      NonFungibleAssets::do_create_attribute(nfa_id, Some(1111), a),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn do_create_attribute_already_exists() {
  new_test_ext().execute_with(|| {
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    // create class
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create fake attr w/ same name
    let eat = AttributeValue::Number(NumberAttribute {
      number_value: 10,
      number_max: None,
    });
    let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    ClassAttributes::<Test>::insert(nfa_id, attr_name, eat);

    assert_noop!(
      NonFungibleAssets::do_create_attribute(nfa_id, Some(org), a),
      Error::<Test>::AttributeAlreadyExists
    );
  });
}

#[test]
fn do_create_attribute_already_exists1() {
  new_test_ext().execute_with(|| {
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    // create class
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().attributes, 0);

    System::reset_events();
    assert_ok!(NonFungibleAssets::do_create_attribute(nfa_id, Some(org), a));

    let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    assert!(ClassAttributes::<Test>::contains_key(nfa_id, &attr_name));
    assert_eq!(Classes::<Test>::get(nfa_id).unwrap().attributes, 1);
    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::AttributeCreated {
          class_id: nfa_id,
          key: attr_name,
          value: AttributeValue::Number(NumberAttribute {
            number_value: 100,
            number_max: None
          })
        }
        .into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn do_remove_attribute_class_not_exists() {
  new_test_ext().execute_with(|| {
    assert_noop!(
      NonFungibleAssets::do_remove_attribute(
        1000.into(),
        None,
        br"a_name".to_vec().try_into().unwrap()
      ),
      Error::<Test>::UnknownClass
    );
  });
}

#[test]
fn do_remove_attribute_owner_no_permissions() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let nfa_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    assert_noop!(
      NonFungibleAssets::do_remove_attribute(
        nfa_id,
        Some(1111),
        br"a_name".to_vec().try_into().unwrap()
      ),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn do_remove_attribute_work() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create attribute
    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };
    assert_ok!(NonFungibleAssets::do_create_attribute(
      class_id,
      Some(org),
      a
    ));
    assert_eq!(Classes::<Test>::get(class_id).unwrap().attributes, 1);
    let key: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    assert_eq!(
      ClassAttributes::<Test>::get(&class_id, &key).unwrap(),
      AttributeValue::Number(NumberAttribute {
        number_value: 100,
        number_max: None,
      })
    );

    System::reset_events();

    assert!(ClassAttributes::<Test>::contains_key(&class_id, &key));
    assert_ok!(NonFungibleAssets::do_remove_attribute(
      class_id,
      Some(org),
      br"a_name".to_vec().try_into().unwrap()
    ));
    assert!(!ClassAttributes::<Test>::contains_key(&class_id, &key));
    assert_eq!(Classes::<Test>::get(class_id).unwrap().attributes, 0);

    assert_eq!(
      System::events(),
      vec![EventRecord {
        phase: Phase::Initialization,
        event: NfaEvent::AttributeRemoved { class_id, key }.into(),
        topics: vec![],
      },]
    );
  });
}

#[test]
fn create_attribute_unsigned() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };

    assert_noop!(
      NonFungibleAssets::create_attribute(Origin::none(), org, class_id, a),
      sp_runtime::traits::BadOrigin
    );
  });
}

#[test]
fn create_attribute_worked() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };

    assert_ok!(NonFungibleAssets::create_attribute(
      Origin::signed(1),
      org,
      class_id,
      a
    ));

    let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    assert!(ClassAttributes::<Test>::contains_key(class_id, &attr_name));
  });
}

#[test]
fn remove_attribute_unsigned() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    let attribute_name = br"a_name".to_vec().try_into().unwrap();

    assert_noop!(
      NonFungibleAssets::remove_attribute(Origin::none(), org, class_id, attribute_name),
      sp_runtime::traits::BadOrigin
    );
  });
}

#[test]
fn remove_attribute_worked() {
  new_test_ext().execute_with(|| {
    // create class
    let name = br"nfa name".to_vec();
    let class_id = get_next_class_id();
    let org = 2;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));

    let a: Attribute = Attribute {
      key: br"a_name".to_vec().try_into().unwrap(),
      value: 100u32.try_into().unwrap(),
    };

    assert_ok!(NonFungibleAssets::create_attribute(
      Origin::signed(1),
      org,
      class_id,
      a.clone()
    ));

    let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
    assert!(ClassAttributes::<Test>::contains_key(class_id, &attr_name));

    assert_ok!(NonFungibleAssets::remove_attribute(
      Origin::signed(1),
      org,
      class_id,
      a.key
    ));
    assert!(!ClassAttributes::<Test>::contains_key(class_id, &attr_name));
  });
}

#[test]
fn assign_attributes_works() {
  new_test_ext().execute_with(|| {
    let a1 = Attribute {
      key: br"a1".to_vec().try_into().unwrap(),
      value: AttributeValue::Number(NumberAttribute {
        number_max: None,
        number_value: 1,
      }),
    };
    let a2 = Attribute {
      key: br"a2".to_vec().try_into().unwrap(),
      value: AttributeValue::Text(br"v1".to_vec().try_into().unwrap()),
    };
    let attributes: AttributeList = vec![a1.clone(), a2.clone()].try_into().unwrap();
    assert_ok!(NonFungibleAssets::assign_attributes(&20.into(), attributes));
    assert_eq!(
      Attributes::<Test>::get(&NonFungibleAssetId::from(20), a1.key),
      Some(a1.value)
    );
    assert_eq!(
      Attributes::<Test>::get(&NonFungibleAssetId::from(20), a2.key),
      Some(a2.value)
    );
  });
}

#[test]
fn set_lock_try_unlock() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::None;
    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.locked, origin);
    assert_noop!(
      NonFungibleAssets::set_lock(&acc, origin, &nfa_id, &id),
      Error::<Test>::Locked
    );
  });
}

#[test]
fn set_lock_unknown_asset() {
  new_test_ext().execute_with(|| {
    let ga = GamerAccount {
      account_id: 1,
      organization_id: 3,
    };
    let origin = Locker::Mechanic(MechanicId {
      gamer_account: ga,
      nonce: 2,
    });
    assert_noop!(
      NonFungibleAssets::set_lock(&1, origin, &234.into(), &123.into()),
      Error::<Test>::UnknownAsset
    );
  });
}

#[test]
fn set_lock_not_owner() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    assert_noop!(
      NonFungibleAssets::set_lock(&5, origin, &nfa_id, &id),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn set_lock_for_no_locked() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    let mut details = Assets::<Test>::get(&nfa_id, &id).unwrap();
    details.locked = origin.clone();

    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin.clone(), &nfa_id, &id),
      LockResultOf::<Test>::Locked(details.clone())
    );

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.locked, origin);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin, &nfa_id, &id),
      LockResultOf::<Test>::Already(details)
    );
  });
}

#[test]
fn set_lock_for_locked() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    let mut details = Assets::<Test>::get(&nfa_id, &id).unwrap();
    details.locked = origin.clone();

    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin.clone(), &nfa_id, &id),
      LockResultOf::<Test>::Locked(details)
    );

    let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
    assert_eq!(minted.locked, origin);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: 2,
        organization_id: 3,
      },
      nonce: 4,
    });
    assert_noop!(
      NonFungibleAssets::set_lock(&acc, origin, &nfa_id, &id),
      Error::<Test>::Locked
    );
  });
}

#[test]
fn unset_lock_for_locked() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    let mut details = Assets::<Test>::get(&nfa_id, &id).unwrap();
    details.locked = origin.clone();

    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin.clone(), &nfa_id, &id),
      LockResultOf::<Test>::Locked(details)
    );

    assert_ok!(NonFungibleAssets::unset_lock(&acc, &origin, &nfa_id, &id));
    let details = Assets::<Test>::get(&nfa_id, &id).unwrap();

    assert_eq!(details.locked, Locker::None);
  });
}

#[test]
fn unset_lock_for_locked_other_owner() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    let mut details = Assets::<Test>::get(&nfa_id, &id).unwrap();
    details.locked = origin.clone();

    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin.clone(), &nfa_id, &id),
      LockResultOf::<Test>::Locked(details)
    );

    assert_noop!(
      NonFungibleAssets::unset_lock(&2, &origin, &nfa_id, &id),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn unset_lock_for_locked_other_origin() {
  new_test_ext().execute_with(|| {
    // create test class
    let nfa_id = get_next_class_id();
    let name = br"nfa name".to_vec();
    let id = get_next_asset_id();
    let org = 2;
    let acc = 1;
    assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name));
    // create test asset
    assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });
    let mut details = Assets::<Test>::get(&nfa_id, &id).unwrap();
    details.locked = origin.clone();

    assert_ok!(
      NonFungibleAssets::set_lock(&acc, origin, &nfa_id, &id),
      LockResultOf::<Test>::Locked(details)
    );
    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 3,
    });
    assert_noop!(
      NonFungibleAssets::unset_lock(&acc, &origin, &nfa_id, &id),
      Error::<Test>::NoPermission
    );
  });
}

#[test]
fn unset_lock_for_unexisted_asset() {
  new_test_ext().execute_with(|| {
    let nfa_id = get_next_class_id();
    let id = get_next_asset_id();
    let acc = 1;
    let org = 2;

    let origin = Locker::Mechanic(MechanicId {
      gamer_account: GamerAccount {
        account_id: acc,
        organization_id: org,
      },
      nonce: 2,
    });

    assert_ok!(NonFungibleAssets::unset_lock(&acc, &origin, &nfa_id, &id));
  });
}
