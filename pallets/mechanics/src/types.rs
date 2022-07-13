use super::*;

/// Type of the non-fungible asset instance ids
pub type NonFungibleAssetId = pallet_support::NonFungibleAssetId;
/// Type of the non-fungible class of assets ids
pub type NonFungibleClassId = pallet_support::NonFungibleClassId;
/// Type of the fungible asset id
pub type FungibleAssetId = pallet_support::FungibleAssetId;
/// The units in which we record balances of the fungible assets
pub type FungibleAssetBalance = pallet_support::FungibleAssetBalance;

/// Bounded vector of NFA ids
pub type NonFungibleAssetIds<T> = BoundedVec<NonFungibleAssetId, <T as Config>::AssetsListLimit>;
/// Describes types of mechanics
pub enum MechanicType {
    /// Creating an NFA
    CreateNFA,
    /// Destroying an NFA
    DestroyNFA,
}

/// Structure to represent of the Mechanic Id
pub struct  MechanicId<T: Config> {
    pub account_id: T::AccountId,
    pub nonce: T::NonceIndex,
}
impl<T: Config> MechanicId<T> {
  /// Creates mechanic id from an account id
  pub fn from_account_id(account_id: T::AccountId) -> MechanicId<T> where
    <T as pallet::Config>::NonceIndex: From<<T as frame_system::Config>::Index>
  {
    let nonce = <frame_system::Pallet<T>>::account_nonce(&account_id);
    MechanicId {
      account_id,
      nonce: nonce.into(),
    }
  }
}
