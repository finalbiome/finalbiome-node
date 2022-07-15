// use super::*;

// use frame_support::{assert_noop, assert_ok};

use crate::{AttributeValue, NumberAttribute};

#[test]
fn template_test() {
  assert_eq!(true, true);
}

#[test]
fn attribute_value_from_u32() {
  let val: AttributeValue = 15u32.try_into().unwrap();
  if let AttributeValue::Number(NumberAttribute { number_value: v, number_max: _ }) = val {
    assert_eq!(v, 15u32);
  } else {
    assert_eq!(false, true);
  }
}

#[test]
fn attribute_value_from_tuple_u32() {
  let val: AttributeValue = (15u32, 100u32).try_into().unwrap();
  if let AttributeValue::Number(NumberAttribute { number_value: v, number_max: m }) = val {
    assert_eq!(v, 15u32);
    assert_eq!(m, Some(100u32))
  } else {
    assert_eq!(false, true);
  }
}

#[test]
#[should_panic(expected = "Attribute numeric value exceeds the maximum value")]
fn attribute_value_from_tuple_u32_error() {
  // hiding trace stack
  std::panic::set_hook(Box::new(|_| {}));
  
  let _: AttributeValue = (150u32, 100u32).try_into().unwrap();
}

#[test]
fn attribute_value_from_str() {
  let val: AttributeValue = "test".try_into().unwrap();
  if let AttributeValue::String(s) = val {
    assert_eq!(s.to_vec(), br"test".to_vec());
  } else {
    assert_eq!(false, true);
  }
}

#[test]
#[should_panic(expected = "String attribute length out of bound")]
fn attribute_value_from_str_error() {
  // hiding trace stack
  std::panic::set_hook(Box::new(|_| {}));

  let _: AttributeValue = "looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________"
    .try_into().unwrap();
}

#[test]
fn attribute_value_from_vec() {
  let val: AttributeValue = br"test".to_vec().try_into().unwrap();
  if let AttributeValue::String(s) = val {
    assert_eq!(s.to_vec(), br"test".to_vec());
  } else {
    assert_eq!(false, true);
  }
}

#[test]
#[should_panic(expected = "String attribute length out of bound")]
fn attribute_value_from_vec_error() {
  // hiding trace stack
  std::panic::set_hook(Box::new(|_| {}));
  
  let _: AttributeValue = br"looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________looooong___string_________"
    .to_vec().try_into().unwrap();
}
