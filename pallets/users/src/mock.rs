use crate as users;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, GenesisBuild, Hooks};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
  testing::Header,
  traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
  AccountId32, MultiSignature,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

type Balance = u64;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system,
    Users: users,
    Balances: pallet_balances,
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
  type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = Event;
  type BlockHashCount = ConstU64<250>;
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<Balance>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = ConstU16<42>;
  type OnSetCode = ();
  type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl users::Config for Test {
  type Event = Event;
  type RecoveryPeriod = ConstU64<5>;
  type Currency = Balances;
  type Capacity = ConstU64<100>;
  type NumberOfSlots = ConstU64<5>;
  type AccountsPerSlotLimit = ConstU32<4>;
}

impl pallet_balances::Config for Test {
  type MaxLocks = ConstU32<50u32>;
  type MaxReserves = ();
  type ReserveIdentifier = [u8; 8];
  /// The type for recording an account's balance.
  type Balance = Balance;
  /// The ubiquitous event type.
  type Event = Event;
  type DustRemoval = ();
  type ExistentialDeposit = ConstU64<10>;
  type AccountStore = System;
  type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

// Build test environment by setting the registrar `key` for the Genesis.
pub fn new_test_ext(registrar_key: &AccountId32) -> sp_io::TestExternalities {
  let mut t = frame_system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();

  users::GenesisConfig::<Test> {
    registrar_key: Some(registrar_key.clone()),
  }
  .assimilate_storage(&mut t)
  .unwrap();
  t.into()
}

/// Progress to the given block.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub fn run_to_block(n: u64) {
  while System::block_number() < n {
    Users::on_finalize(System::block_number());
    System::on_finalize(System::block_number());

    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    Users::on_initialize(System::block_number());
  }
}
