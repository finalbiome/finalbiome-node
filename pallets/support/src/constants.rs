use super::*;

/// Lenght limit of the string attribute value
pub type AttributeValueStringLimit = ConstU32<64>;
/// Lenght limit of the string attribute key
pub type AttributeKeyStringLimit = ConstU32<32>;
/// Maximum capacity of attribute lists
pub type AttributeListLengthLimit = ConstU32<{ u8::MAX as u32 }>;
/// Default length of the string data type
pub type DefaultStringLimit = ConstU32<64>;

pub(crate) const ERROR_VALIDATE_NUMBER_ATTRIBUTE: &str = "Attribute numeric value exceeds the maximum value";
pub(crate) const ERROR_VALIDATE_TEXT_ATTRIBUTE: &str = "String attribute length out of bound";
