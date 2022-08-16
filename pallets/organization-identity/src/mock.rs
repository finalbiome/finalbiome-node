use crate as pallet_organization_identity;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use pallet_support::{DispatchResultAs, Locker, NonFungibleClassId, NonFungibleAssetId, LockResult, types_nfa::ClassDetails, FungibleAssetId, FungibleAssetBalance};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup}, DispatchError,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		OrganizationIdentity: pallet_organization_identity::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u32;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// Mock of fungible-assets-pallet impl
pub struct FAPallet {}
impl pallet_support::traits::FungibleAssets<u64> for FAPallet {
fn can_withdraw(
		_asset: u32,
		_who: &u64,
		_amount: u128,
	) -> frame_support::traits::tokens::WithdrawConsequence<u128> {
    todo!()
  }

	fn burn_from(
		_asset_id: u32, 
		_who: &u64, 
		_amount: u128,
	) -> pallet_support::DispatchResultAs<u128> {
		todo!()
	}

	fn inc_references(_asset: &FungibleAssetId) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn dec_references(_asset: &FungibleAssetId) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn mint_into(asset: FungibleAssetId, _who: &u64, amount: FungibleAssetBalance) -> sp_runtime::DispatchResult {
		if asset == 10 { // test do_airdrop_fa_works
			return Ok(())
		}
		if asset == 11 { // test do_airdrop_fa_works
			return Err(DispatchError::Other("mock_do_airdrop_fa_works"))
		}
		if asset == 12 { // test do_onboarding_works, onboarding_works
			assert!(amount == 100);
			return Ok(())
		}
		todo!()
	}
}

/// Mock of non-fungible-assets-pallet impl
pub struct NFAPallet {}
impl pallet_support::traits::NonFungibleAssets<u64, u32> for NFAPallet {

fn mint_into(
    class_id: &u32,
    who: &u64
  ) -> DispatchResultAs<u32> {
		if class_id == &10 { // test do_airdrop_nfa_works
			return Ok(10)
		}
		if class_id == &11 { // test do_airdrop_nfa_works
			return Err(DispatchError::Other("mock_do_airdrop_nfa_works"))
		}
		if class_id == &12 { // test do_airdrop_nfa_works
			return Ok(12)
		}
		if class_id == &13 { // test do_onboarding_works, onboarding_works
			assert!(who == &333);
			return Ok(13)
		}
		todo!()
	}

	fn get_offer(
    _class_id: &pallet_support::NonFungibleClassId,
    _offer_id: &u32,
  ) -> pallet_support::DispatchResultAs<(pallet_support::FungibleAssetId, pallet_support::FungibleAssetBalance, pallet_support::AttributeList)> {
		todo!()
	}

	fn set_attributes(
    asset_id: &pallet_support::NonFungibleAssetId,
    attributes: pallet_support::AttributeList,
  ) -> frame_support::dispatch::DispatchResult {
		if asset_id == &10 { // test do_airdrop_nfa_works
			return Ok(())
		}
		if asset_id == &12 { // test do_airdrop_nfa_works
			return Err(DispatchError::Other("mock_do_airdrop_nfa_works_2"))
		}
		if asset_id == &13 { // test do_onboarding_works, onboarding_works
			assert!(attributes.get(0).unwrap().value == 10.try_into().unwrap());
			assert!(attributes.get(1).unwrap().value == "t".try_into().unwrap());
			return Ok(())
		}
		todo!()
	}
	fn try_lock(
		_who: &u64,
		_origin: Locker<u64, u32>,
		_class_id: &NonFungibleClassId,
		_asset_id: &NonFungibleAssetId,
	) -> DispatchResultAs<LockResult<u64, u32>> {
		todo!()
	}
	fn get_class(
		_class_id: &NonFungibleClassId,
	) -> DispatchResultAs<ClassDetails<u64>> {
		todo!()
	}
	fn burn(
		_class_id: NonFungibleClassId,
		_asset_id: NonFungibleAssetId,
		_maybe_check_owner: Option<&u64>,
	) -> sp_runtime::DispatchResult {
		todo!()
	}
	fn clear_lock(
		_who: &u64,
		_origin: &Locker<u64, u32>,
		_class_id: &NonFungibleClassId,
		_asset_id: &NonFungibleAssetId,
	) -> sp_runtime::DispatchResult {
		todo!()
	}
}


impl pallet_organization_identity::Config for Test {
	type Event = Event;
	type FungibleAssets = FAPallet;
	type NonFungibleAssets = NFAPallet;
	type ExecuteOrigin = frame_system::EnsureSigned<u64>;
	type StringLimit = frame_support::traits::ConstU32<10>;
	type MaxMembers = frame_support::traits::ConstU8<3>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		// System::on_initialize(1);
		// NonFungibleAssets::on_initialize(1);
	});
	ext
}
