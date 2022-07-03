use super::*;

/// Type of the non-fungible asset ids
pub type AssetId = u32;

/// Type of the non-fungible class of assets ids
pub type ClassId = u32;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ClassDetails<AccountId, BoundedString> {
  pub(super) owner: AccountId,
  /// The total number of outstanding instances of this asset class
	pub(super) instances: u32,
  /// Name of the Asset. Limited in length by `ClassNameLimit`
	pub(super) name: BoundedString,
}

#[derive(RuntimeDebug, PartialEq)]
pub struct ClassDetailsBuilder<T: Config> {
  owner: T::AccountId,
  name: ClassNameLimit<T>,
}
impl<T: pallet::Config> ClassDetailsBuilder<T> {
  pub fn new(owner: T::AccountId, name: Vec<u8>) -> ClassDetailsBuilderResult<T> {
    let name = name.try_into();
    let name = match name {
      Ok(name) => name,
      Err(_) => return Err(Error::<T>::ClassNameTooLong.into()),
    };
    Ok(ClassDetailsBuilder {
      owner,
      name
    })
  }

  /// Validation of the all class details.
  fn validate(&self) -> DispatchResult {
    Ok(())
  }

  pub fn build(self) -> Result<ClassDetails<T::AccountId, ClassNameLimit<T>>, DispatchError> {
    self.validate()?;
    Ok(ClassDetails {
      owner: self.owner,
      name: self.name,
      instances: Zero::zero(),
    })
  }
}

pub type ClassNameLimit<T> = BoundedVec<u8, <T as pallet::Config>::ClassNameLimit>;
type ClassDetailsBuilderResult<T> = Result<ClassDetailsBuilder<T>, DispatchError>;
