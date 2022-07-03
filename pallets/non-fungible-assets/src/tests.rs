use super::*;

use crate::{
	mock::*, Error,
	ClassDetailsBuilder,
};
use frame_support::{assert_noop, assert_ok};

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
	});
}

#[test]
fn class_details_builder_name_len_exceed() {
	new_test_ext().execute_with(|| {
		assert_noop!(ClassDetailsBuilder::<Test>::new(1, br"n234567810".to_vec()), Error::<Test>::ClassNameTooLong); // max 8 symbols
	});
}

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(NonFungibleAssets::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(NonFungibleAssets::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(NonFungibleAssets::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}
