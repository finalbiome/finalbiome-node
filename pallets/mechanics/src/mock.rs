use crate as pallet_mechanics;
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
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
		MechanicsModule: pallet_mechanics::{Pallet, Call, Storage, Event<T>},
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
impl support::FungibleAssets<u64> for FAPallet {
fn can_withdraw(
		asset: u32,
		who: &u64,
		amount: u128,
	) -> frame_support::traits::tokens::WithdrawConsequence<u128> {
        todo!()
    }

fn burn_from(
    asset: u32, 
    who: &u64, 
    amount: u128,
  ) -> support::DispatchResultAs<u128> {
        todo!()
    }
}

/// Mock of non-fungible-assets-pallet impl
pub struct NFAPallet {}
impl support::NonFungibleAssets<u64> for NFAPallet {

fn mint_into(
    class_id: &u32,
    who: &u64
  ) -> frame_support::dispatch::DispatchResult {
        todo!()
    }

fn get_offer(
    class_id: &support::NonFungibleClassId,
    offer_id: &u32,
  ) -> support::DispatchResultAs<(support::FungibleAssetId, support::FungibleAssetBalance)> {
        todo!()
    }
}

impl pallet_mechanics::Config for Test {
	type Event = Event;
	type FungibleAssets = FAPallet;
	type NonFungibleAssets = NFAPallet;
	type NonceIndex = u32;
	type AssetsListLimit = ConstU32<16>;
	type MechanicsLifeTime = ConstU64<20>;
}

// Build genesis storage according to the mock runtime.
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
