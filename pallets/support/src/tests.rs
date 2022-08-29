// use super::*;

use frame_support::{assert_ok};

use crate::{AttributeValue, NumberAttribute, bettor::*, purchased::*, characteristics::*, misc::{cumsum_array_owned, cumsum_owned}, BETTOR_MAX_NUMBER_OF_ROUNDS, };

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
fn attribute_value_from_opt_tuple_u32() {
  let val: AttributeValue = (15u32, Some(100u32)).try_into().unwrap();
  if let AttributeValue::Number(NumberAttribute { number_value: v, number_max: m }) = val {
    assert_eq!(v, 15u32);
    assert_eq!(m, Some(100u32))
  } else {
    assert_eq!(false, true);
  }
  let val: AttributeValue = (15u32, None).try_into().unwrap();
  if let AttributeValue::Number(NumberAttribute { number_value: v, number_max: m }) = val {
    assert_eq!(v, 15u32);
    assert_eq!(m, None)
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
  if let AttributeValue::Text(s) = val {
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
  if let AttributeValue::Text(s) = val {
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

#[test]
fn attribute_validate_w_max_val() {
  let val: AttributeValue = AttributeValue::Number(NumberAttribute {
    number_value: 10,
    number_max: None,
  });
  assert_ok!(val.validate());
}

#[test]
fn attribute_validate_no_max_val() {
  let val: AttributeValue = AttributeValue::Number(NumberAttribute {
    number_value: 10,
    number_max: Some(100),
  });
  assert_ok!(val.validate());
}
#[test]
#[should_panic(expected = "Attribute numeric value exceeds the maximum value")]
fn attribute_validate_no_max_val_err() {
  // hiding trace stack
  std::panic::set_hook(Box::new(|_| {}));
    
  let val: AttributeValue = AttributeValue::Number(NumberAttribute {
    number_value: 10,
    number_max: Some(1),
  });
  val.validate().unwrap();
}

#[test]
fn bettor_empty() {
  let b:Bettor = Bettor {
    outcomes: vec![].try_into().expect("Outcomes vec too big"),
    winnings: vec![].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false)
}

#[test]
fn bettor_probs_eq_0() {
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 0,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out1".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Lose,
      },
      BettorOutcome {
        name: br"out2".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Draw,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true);
}

#[test]
fn bettor_outcomes_less_2() {
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out1".to_vec().try_into().expect("too long"),
        probability: 100,
        result: OutcomeResult::Win,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Win,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Lose,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true);
}

#[test]
fn bettor_rounds_less_1() {
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 0,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true);
}

#[test]
fn bettor_rounds_more_than_limit() {
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: BETTOR_MAX_NUMBER_OF_ROUNDS + 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: BETTOR_MAX_NUMBER_OF_ROUNDS,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true);
}

#[test]
fn bettor_wins_empty() {
  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out1".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);

  let b:Bettor = Bettor {
    outcomes: vec![
      BettorOutcome {
        name: br"out0".to_vec().try_into().expect("too long"),
        probability: 5,
        result: OutcomeResult::Win,
      },
      BettorOutcome {
        name: br"out1".to_vec().try_into().expect("too long"),
        probability: 95,
        result: OutcomeResult::Lose,
      },
    ].try_into().expect("Outcomes vec too big"),
    winnings: vec![
      BettorWinning::Fa(1.into(), 33.into()),
    ].try_into().expect("Winnings vec too big"),
    rounds: 1,
    draw_outcome: DrawOutcomeResult::Keep,
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true);
}

#[test]
fn purchased_empty() {
  let b:Purchased = Purchased {
    offers: vec![].try_into().unwrap(),
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false);
}

#[test]
fn purchased_has_0_price() {
  let b:Purchased = Purchased {
    offers: vec![
      Offer {
        fa: 1.into(),
        price: 10.into(),
        attributes: vec![].try_into().unwrap(),
      },
      Offer {
        fa: 2.into(),
        price: 100.into(),
        attributes: vec![].try_into().unwrap(),
      },
      Offer {
        fa: 3.into(),
        price: 0.into(),
        attributes: vec![].try_into().unwrap(),
      },
    ].try_into().unwrap(),
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), false)
}

#[test]
fn purchased_has_0_price_2() {
  let b:Purchased = Purchased {
    offers: vec![
      Offer {
        fa: 1.into(),
        price: 10.into(),
        attributes: vec![].try_into().unwrap(),
      },
      Offer {
        fa: 2.into(),
        price: 100.into(),
        attributes: vec![].try_into().unwrap(),
      },
      Offer {
        fa: 3.into(),
        price: 1000.into(),
        attributes: vec![].try_into().unwrap(),
      },
    ].try_into().unwrap(),
  };
  assert_eq!(AssetCharacteristic::is_valid(&b), true)
}

#[test]
fn test_cumsums() {
  let a: [i32; 0] = [];
  assert_eq!(cumsum_array_owned::<i32, 0>([]), a);
  assert_eq!(cumsum_array_owned([1]), [1]);
  assert_eq!(cumsum_array_owned([1, 2, 3]), [1, 3, 6]);

  // assert_eq!(cumsum_owned::<i32>(vec![]), a.into::<i32>());
  assert_eq!(cumsum_owned(vec![1]), vec![1]);
  assert_eq!(cumsum_owned(vec![1, 2, 3]), vec![1, 3, 6]);
}
