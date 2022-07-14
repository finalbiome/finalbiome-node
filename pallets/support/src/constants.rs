use super::*;

/// Lenght limit of the string attribute value
pub type AttributeValueStringLimit = ConstU32<64>;
/// Lenght limit of the string attribute key
pub type AttributeKeyStringLimit = ConstU32<32>;
/// Maximum capacity of attribute lists
pub type AttributeListLengthLimit = ConstU32<{ u8::MAX as u32 }>;
