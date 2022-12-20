//! Various basic types for use in the assets pallet.
#[cfg(feature = "std")]
use sp_std::vec::Vec;

use frame_support::pallet_prelude::*;
use pallet_support::{
  AttributeList, AttributeListLengthLimit, FungibleAssetBalance, FungibleAssetId,
  NonFungibleClassId,
};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct OrganizationDetails<BoundedString> {
  /// Name of the Organization. Limited in length by `StringLimit`.
  pub name: BoundedString,
  pub onboarding_assets: OnboardingAssets,
}

impl<BoundedString> OrganizationDetails<BoundedString> {
  pub fn new(name: BoundedString) -> OrganizationDetails<BoundedString> {
    OrganizationDetails {
      name,
      onboarding_assets: Default::default(),
    }
  }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// Represents an asset which will be airdropped on game onboarding
pub enum AirDropAsset {
  Fa(FungibleAssetId, FungibleAssetBalance),
  Nfa(NonFungibleClassId, AttributeList),
}

/// Type alias for `frame_system`'s account id as an organization id.
pub type OrganizationIdOf<T> = <T as frame_system::Config>::AccountId;
pub type OnboardingAssets = Option<BoundedVec<AirDropAsset, AttributeListLengthLimit>>;

// region: Genesis Types
#[cfg(feature = "std")]
pub type GenesisOnboardingFaConfigOf<T> = Vec<(
  OrganizationIdOf<T>,
  Vec<(FungibleAssetId, FungibleAssetBalance)>,
)>;
// endregion: Genesis Types
