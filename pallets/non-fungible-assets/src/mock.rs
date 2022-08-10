use crate as pallet_non_fungible_assets;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, AsEnsureOriginWithArg};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use pallet_support;

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
		NonFungibleAssets: pallet_non_fungible_assets::{Pallet, Call, Storage, Event<T>},
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
	type MaxConsumers = ConstU32<16>;
}

/// Mock of fungible-assets-pallet impl
pub struct FAPallet {}
impl pallet_support::traits::FungibleAssets<u64> for FAPallet {
	fn can_withdraw(
		_asset: pallet_support::FungibleAssetId,
		_who: &u64,
		_amount: pallet_support::FungibleAssetBalance,
	) -> frame_support::traits::tokens::WithdrawConsequence<pallet_support::FungibleAssetBalance> {
		todo!()
	}

	fn burn_from(
    _asset: pallet_support::FungibleAssetId, 
    _who: &u64, 
    _amount: pallet_support::FungibleAssetBalance
  ) -> pallet_support::DispatchResultAs<pallet_support::FungibleAssetBalance> {
		todo!()
	}
	
	fn inc_references(_asset: &pallet_support::FungibleAssetId) -> sp_runtime::DispatchResult {
		Ok(())
	}
	fn dec_references(_asset: &pallet_support::FungibleAssetId) -> sp_runtime::DispatchResult {
		Ok(())
	}
	fn mint_into(_asset: pallet_support::FungibleAssetId, _who: &u64, _amount: pallet_support::FungibleAssetBalance) -> sp_runtime::DispatchResult {
		todo!()
	}
}

impl pallet_non_fungible_assets::Config for Test {
	type Event = Event;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type FungibleAssets = FAPallet;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	// let config: pallet_fungible_assets::GenesisConfig<Test> = pallet_fungible_assets::GenesisConfig {
	// 	// assets: asset_id, organization_id, name, top_upped_speed, cup_global, cup_local
	// 	assets: vec![
	// 		(0, 2, "asset01".into(), None, None, None),
	// 		(1, 2, "asset02".into(), Some(5), None, Some(20)),
	// 	],
	// 	// account_balances: account_id, asset_id, balance
	// 	accounts: vec![
	// 		(1, 0, 1_000),
	// 		(3, 1, 20),
	// 		(4, 1, 5),
	// 		(5, 0, 10_000),
	// 	],
	// };
	// config.assimilate_storage(&mut storage).unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		// System::on_initialize(1);
		// NonFungibleAssets::on_initialize(1);
	});
	ext
}
