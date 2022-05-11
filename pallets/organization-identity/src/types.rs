//! Various basic types for use in the assets pallet.
// use super::*;

use frame_support::{
	pallet_prelude::*,
};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct OrganizationDetails<BoundedString> {
  /// Name of the Organization. Limited in length by `StringLimit`.
  pub(super) name: BoundedString,
}

impl<BoundedString> OrganizationDetails<BoundedString> {
  pub fn new(name: BoundedString) -> OrganizationDetails<BoundedString> {
    OrganizationDetails { name }
  }  
}

/// Type alias for `frame_system`'s account id as an organization id.
pub type OrganizationIdOf<T> = <T as frame_system::Config>::AccountId;
