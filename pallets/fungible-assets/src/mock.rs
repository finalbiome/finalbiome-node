use crate as pallet_fungible_assets;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, AsEnsureOriginWithArg, GenesisBuild};
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
		FungibleAssets: pallet_fungible_assets::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
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

impl pallet_fungible_assets::Config for Test {
	type Event = Event;
	type Balance = u64;
	// type CreateOrigin = frame_system::EnsureRoot<u64>;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type OrganizationId = u64;
	type NameLimit = ConstU32<8>;
	// type MaxAssets = ConstU32<6>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let config: pallet_fungible_assets::GenesisConfig<Test> = pallet_fungible_assets::GenesisConfig {
		// assets: asset_id, organization_id, name, top_upped_speed, cup_global, cup_local
		assets: vec![
			(0, 2, "asset01".into(), None, None, None),
			(1, 2, "asset02".into(), Some(5), None, Some(20)),
		],
		// account_balances: asset_id, account_id, balance
		accounts: vec![
			(0, 1, 1000),
			(1, 3, 20),
		],
	};
	config.assimilate_storage(&mut storage).unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub type SysEvent = frame_system::Event<Test>;
