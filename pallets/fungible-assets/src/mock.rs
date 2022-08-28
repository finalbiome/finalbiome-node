use crate as pallet_fungible_assets;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, AsEnsureOriginWithArg, GenesisBuild, Hooks};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type BlockNumber = u64;

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
	type BlockNumber = BlockNumber;
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
	// type CreateOrigin = frame_system::EnsureRoot<u64>;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type OrganizationId = u64;
	type NameLimit = ConstU32<8>;
	type MaxTopUppedAssets = ConstU32<{ u32::MAX }>;
	// type MaxAssets = ConstU32<6>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let config: pallet_fungible_assets::GenesisConfig<Test> = pallet_fungible_assets::GenesisConfig {
		// assets: asset_id, organization_id, name, top_upped_speed, cup_global, cup_local
		assets: vec![
			(0.into(), 2, "asset01".into(), None, None, None),
			(1.into(), 2, "asset02".into(), Some(5), None, Some(20)),
		],
		// account_balances: account_id, asset_id, balance
		accounts: vec![
			(1, 0.into(), 1_000),
			(3, 1.into(), 20),
			(4, 1.into(), 5),
			(5, 0.into(), 10_000),
		],
	};
	config.assimilate_storage(&mut storage).unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		System::on_initialize(1);
		FungibleAssets::on_initialize(1);
	});
	ext
}

/// Progress to the given block.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub fn run_to_block(n: BlockNumber) {
	while System::block_number() < n {
		FungibleAssets::on_finalize(System::block_number());
  	System::on_finalize(System::block_number());

		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		FungibleAssets::on_initialize(System::block_number());
	}
}

pub type SysEvent = frame_system::Event<Test>;
