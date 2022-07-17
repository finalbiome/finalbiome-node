use super::*;

use crate::{
	mock::*, Error,
	ClassDetailsBuilder,
	Event as NfaEvent,
	characteristics::{bettor::*, purchased::{Purchased, Offer}},
};
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};

fn get_next_class_id() -> u32 {
	NextClassId::<Test>::get()
}

fn get_next_asset_id() -> u32 {
	NextAssetId::<Test>::get()
}

#[test]
fn template_test() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn next_class_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 0);
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 1);
		assert_eq!(NonFungibleAssets::get_next_class_id().unwrap(), 2);
		assert_eq!(NextClassId::<Test>::get(), 3);
	});
}

#[test]
fn next_asset_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 0);
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 1);
		assert_eq!(NonFungibleAssets::get_next_asset_id().unwrap(), 2);
		assert_eq!(NextAssetId::<Test>::get(), 3);
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
		assert_noop!(ClassDetailsBuilder::<Test>::new(1, br"n234567810".to_vec()), Error::<Test>::ClassNameTooLong); // max 8 symbols
	});
}

#[test]
fn asset_details_builder() {
	new_test_ext().execute_with(|| {
		let b = AssetDetailsBuilder::<Test>::new(1).unwrap();
		let d = b.build().unwrap();
		assert_eq!(d.owner, 1);
	});
}

#[test]
fn create_class_unsigned() {
	new_test_ext().execute_with(|| {
		let name = br"nfa name".to_vec();
		assert_noop!(NonFungibleAssets::create(Origin::none(), 2, name), sp_runtime::traits::BadOrigin);
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
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), true);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::Created { class_id: nfa_id, owner: org }.into(),
					topics: vec![],
				},
			]
		);

	});
}

#[test]
fn do_destroy_class_unknown_class() {
	new_test_ext().execute_with(|| {
		assert_noop!(NonFungibleAssets::do_destroy_class(888, Some(999)), Error::<Test>::UnknownClass);
	});
}

#[test]
fn do_destroy_class_not_owner() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		assert_noop!(NonFungibleAssets::do_destroy_class(nfa_id, Some(3)), Error::<Test>::NoPermission);
		assert_eq!(Classes::<Test>::contains_key(nfa_id), true);
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), true);
	});
}

#[test]
fn do_destroy_class_worked() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		System::reset_events();
		assert_ok!(NonFungibleAssets::do_destroy_class(nfa_id, Some(org)));
		assert_eq!(Classes::<Test>::contains_key(nfa_id), false);
		assert_eq!(ClassAccounts::<Test>::contains_key(org, nfa_id), false);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::Destroyed { class_id: nfa_id }.into(),
					topics: vec![],
				},
			]
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
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		// create attribute
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		assert_ok!(NonFungibleAssets::do_create_attribute(class_id, Some(org), a));
		let key: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		assert_eq!(Attributes::<Test>::contains_key((&class_id, None as Option<NonFungibleAssetId>, &key)), true);
		assert_eq!(Classes::<Test>::get(&class_id).unwrap().attributes, 1);

		assert_ok!(NonFungibleAssets::do_destroy_class(class_id, Some(org)));
		assert_eq!(Classes::<Test>::contains_key(&class_id), false);
		assert_eq!(ClassAccounts::<Test>::contains_key(&org, &class_id), false);
		assert_eq!(Attributes::<Test>::contains_key((&class_id, None as Option<NonFungibleAssetId>, &key)), false);

	});
}

#[test]
fn destroy_class_not_org() {
	new_test_ext().execute_with(|| {
		// create test asset
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		System::reset_events();
		assert_noop!(NonFungibleAssets::destroy(Origin::none(), org, nfa_id), sp_runtime::traits::BadOrigin);
	});
}

#[test]
fn bettor_empty() {
	new_test_ext().execute_with(|| {
		let b:Bettor = Bettor {
			outcomes: vec![].try_into().expect("Outcomes vec too big"),
			winnings: vec![].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn bettor_prob_more_100() {
	new_test_ext().execute_with(|| {
		let b:Bettor = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 233,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::Fa(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn bettor_probs_less_100() {
	new_test_ext().execute_with(|| {
		let b:Bettor = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::Fa(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false);

		let b:Bettor = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 100,
				}
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::Fa(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), true);
	});
}

#[test]
fn bettor_wins_empty() {
	new_test_ext().execute_with(|| {
		let b:Bettor = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				},
				BettorOutcome {
					name: br"out1".to_vec().try_into().expect("too long"),
					probability: 95,
				},
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), false);

		let b:Bettor = Bettor {
			outcomes: vec![
				BettorOutcome {
					name: br"out0".to_vec().try_into().expect("too long"),
					probability: 5,
				},
				BettorOutcome {
					name: br"out1".to_vec().try_into().expect("too long"),
					probability: 95,
				},
			].try_into().expect("Outcomes vec too big"),
			winnings: vec![
				BettorWinning::Fa(1, 33),
			].try_into().expect("Winnings vec too big"),
			rounds: 1,
			draw_outcome: DrawOutcomeResult::Keep,
		};
		assert_eq!(b.is_valid(), true);
	});
}

#[test]
fn do_mint_class_unknown_class() {
	new_test_ext().execute_with(|| {
		assert_noop!(NonFungibleAssets::do_mint(888, 999), Error::<Test>::UnknownClass);
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
		assert_ok!(NonFungibleAssets::create(
			Origin::signed(1),
			org,
			name.clone()
		));
		System::reset_events();
		assert_eq!(NonFungibleAssets::do_mint(nfa_id, acc).unwrap(), id);
		assert_eq!(Assets::<Test>::contains_key(&nfa_id, &id), true);
		assert_eq!(Accounts::<Test>::contains_key((&acc, &nfa_id, &id)), true);
		assert_eq!(Classes::<Test>::get(nfa_id).unwrap().instances, 1);
		
		let minted = Assets::<Test>::get(&nfa_id, &id).unwrap();
		assert_eq!(minted.owner, acc);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::Issued { class_id: nfa_id, asset_id: id, owner: acc }.into(),
					topics: vec![],
				},
			]
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
		assert_noop!(NonFungibleAssets::do_create_attribute(1, None, a), DispatchError::Other("Attribute numeric value exceeds the maximum value"));
	});
}

#[test]
fn do_create_attribute_class_not_exists() {
	new_test_ext().execute_with(|| {
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		assert_noop!(NonFungibleAssets::do_create_attribute(1000, None, a), Error::<Test>::UnknownClass);
	});
}

#[test]
fn do_create_attribute_owner_no_permissions() {
	new_test_ext().execute_with(|| {
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		// create class
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		assert_noop!(NonFungibleAssets::do_create_attribute(nfa_id, Some(1111), a), Error::<Test>::NoPermission);
	});
}

#[test]
fn do_create_attribute_already_exists() {
	new_test_ext().execute_with(|| {
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		// create class
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));
		// create fake attr w/ same name
		let eat = AttributeValue::Number(NumberAttribute {number_value: 10, number_max: None});
		let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		Attributes::<Test>::insert((nfa_id, None as Option<NonFungibleAssetId>, attr_name), eat);

		assert_noop!(NonFungibleAssets::do_create_attribute(nfa_id, Some(org), a), Error::<Test>::AttributeAlreadyExists);
	});
}

#[test]
fn do_create_attribute_already_exists1() {
	new_test_ext().execute_with(|| {
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		// create class
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));
		assert_eq!(Classes::<Test>::get(nfa_id).unwrap().attributes, 0);

		System::reset_events();
		assert_ok!(NonFungibleAssets::do_create_attribute(nfa_id, Some(org), a));
		
		let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		assert_eq!(Attributes::<Test>::contains_key((nfa_id, None as Option<NonFungibleAssetId>, &attr_name)), true);
		assert_eq!(Classes::<Test>::get(nfa_id).unwrap().attributes, 1);
		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::AttributeCreated { class_id: nfa_id, key: attr_name, value: AttributeValue::Number(NumberAttribute {number_value: 100, number_max: None}) }.into(),
					topics: vec![],
				},
			]
		);
	});
}

#[test]
fn do_remove_attribute_class_not_exists() {
	new_test_ext().execute_with(|| {
		assert_noop!(NonFungibleAssets::do_remove_attribute(1000, None, br"a_name".to_vec().try_into().unwrap()), Error::<Test>::UnknownClass);
	});
}

#[test]
fn do_remove_attribute_owner_no_permissions() {
	new_test_ext().execute_with(|| {
		// create class
		let name = br"nfa name".to_vec();
		let nfa_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		assert_noop!(NonFungibleAssets::do_remove_attribute(nfa_id, Some(1111), br"a_name".to_vec().try_into().unwrap()), Error::<Test>::NoPermission);
	});
}

#[test]
fn do_remove_attribute_work() {
	new_test_ext().execute_with(|| {
		// create class
		let name = br"nfa name".to_vec();
		let class_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));
		// create attribute
		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};
		assert_ok!(NonFungibleAssets::do_create_attribute(class_id, Some(org), a));
		assert_eq!(Classes::<Test>::get(class_id).unwrap().attributes, 1);
		let key: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		assert_eq!(Attributes::<Test>::get((&class_id, None as Option<NonFungibleAssetId>, &key)).unwrap(), AttributeValue::Number(NumberAttribute {
			number_value: 100,
			number_max: None,
		}));

		System::reset_events();

		assert_eq!(Attributes::<Test>::contains_key((&class_id, None as Option<NonFungibleAssetId>, &key)), true);
		assert_ok!(NonFungibleAssets::do_remove_attribute(class_id, Some(org), br"a_name".to_vec().try_into().unwrap()));
		assert_eq!(Attributes::<Test>::contains_key((&class_id, None as Option<NonFungibleAssetId>, &key)), false);
		assert_eq!(Classes::<Test>::get(class_id).unwrap().attributes, 0);

		assert_eq!(
			System::events(),
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: NfaEvent::AttributeRemoved { class_id, key }.into(),
					topics: vec![],
				},
			]
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
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};

		assert_noop!(NonFungibleAssets::create_attribute(Origin::none(), org, class_id, a), sp_runtime::traits::BadOrigin);
	});
}

#[test]
fn create_attribute_worked() {
	new_test_ext().execute_with(|| {
		// create class
		let name = br"nfa name".to_vec();
		let class_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};

		assert_ok!(NonFungibleAssets::create_attribute(Origin::signed(1), org, class_id, a));
		
		let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		assert_eq!(Attributes::<Test>::contains_key((class_id, None as Option<NonFungibleAssetId>, &attr_name)), true);

	});
}

#[test]
fn remove_attribute_unsigned() {
	new_test_ext().execute_with(|| {
		// create class
		let name = br"nfa name".to_vec();
		let class_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		let attribute_name = br"a_name".to_vec().try_into().unwrap();

		assert_noop!(NonFungibleAssets::remove_attribute(Origin::none(), org, class_id, attribute_name), sp_runtime::traits::BadOrigin);
	});
}

#[test]
fn remove_attribute_worked() {
	new_test_ext().execute_with(|| {
		// create class
		let name = br"nfa name".to_vec();
		let class_id = get_next_class_id();
		let org = 2;
		assert_ok!(NonFungibleAssets::create(Origin::signed(1), org, name.clone()));

		let a: Attribute = Attribute {
			key: br"a_name".to_vec().try_into().unwrap(),
			value: 100u32.try_into().unwrap()
		};

		assert_ok!(NonFungibleAssets::create_attribute(Origin::signed(1), org, class_id, a.clone()));
		
		let attr_name: AttributeKey = br"a_name".to_vec().try_into().unwrap();
		assert_eq!(Attributes::<Test>::contains_key((class_id, None as Option<NonFungibleAssetId>, &attr_name)), true);

		assert_ok!(NonFungibleAssets::remove_attribute(Origin::signed(1), org, class_id, a.key));
		assert_eq!(Attributes::<Test>::contains_key((class_id, None as Option<NonFungibleAssetId>, &attr_name)), false);


	});
}

#[test]
fn purchased_empty() {
	new_test_ext().execute_with(|| {
		let b:Purchased = Purchased {
			offers: vec![].try_into().unwrap(),
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn purchased_has_0_price() {
	new_test_ext().execute_with(|| {
		let b:Purchased = Purchased {
			offers: vec![
				Offer {
					fa: 1,
					price: 10,
					attributes: vec![].try_into().unwrap(),
				},
				Offer {
					fa: 2,
					price: 100,
					attributes: vec![].try_into().unwrap(),
				},
				Offer {
					fa: 3,
					price: 0,
					attributes: vec![].try_into().unwrap(),
				},
			].try_into().unwrap(),
		};
		assert_eq!(b.is_valid(), false)
	});
}

#[test]
fn purchased_has_0_price_2() {
	new_test_ext().execute_with(|| {
		let b:Purchased = Purchased {
			offers: vec![
				Offer {
					fa: 1,
					price: 10,
					attributes: vec![].try_into().unwrap(),
				},
				Offer {
					fa: 2,
					price: 100,
					attributes: vec![].try_into().unwrap(),
				},
				Offer {
					fa: 3,
					price: 1000,
					attributes: vec![].try_into().unwrap(),
				},
			].try_into().unwrap(),
		};
		assert_eq!(b.is_valid(), true)
	});
}

#[test]
fn assign_attributes_works() {
	new_test_ext().execute_with(|| {
		let a1 = Attribute {
			key: br"a1".to_vec().try_into().unwrap(),
			value: AttributeValue::Number(NumberAttribute { number_max: None, number_value: 1})
		};
		let a2 = Attribute {
			key: br"a2".to_vec().try_into().unwrap(),
			value: AttributeValue::Text(br"v1".to_vec().try_into().unwrap())
		};
		let attributes: AttributeList = vec![a1.clone(), a2.clone()].try_into().unwrap();
		assert_ok!(NonFungibleAssets::assign_attributes(&10, &20, attributes));
		assert_eq!(Attributes::<Test>::get((&10, Some(&20), a1.key)), Some(a1.value));
		assert_eq!(Attributes::<Test>::get((&10, Some(&20), a2.key)), Some(a2.value));
	});
}
