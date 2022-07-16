use crate as pallet_mechanics;
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use frame_system as system;
use pallet_support::*;
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
impl pallet_support::traits::FungibleAssets<u64> for FAPallet {
fn can_withdraw(
		asset: u32,
		_who: &u64,
		amount: u128,
	) -> frame_support::traits::tokens::WithdrawConsequence<u128> {
    if amount > 9999 {
			return frame_support::traits::tokens::WithdrawConsequence::NoFunds
		}
		if asset == 333 {
			return frame_support::traits::tokens::WithdrawConsequence::UnknownAsset
		}
		frame_support::traits::tokens::WithdrawConsequence::Success
  }

fn burn_from(
	asset_id: u32, 
    _who: &u64, 
    _amount: u128,
  ) -> pallet_support::DispatchResultAs<u128> {
		if asset_id == 5u32 {
			return Ok(10000u128);
		}
		todo!()
	}
}

/// Mock of non-fungible-assets-pallet impl
pub struct NFAPallet {}
impl pallet_support::traits::NonFungibleAssets<u64> for NFAPallet {

fn mint_into(
    class_id: &u32,
    _who: &u64
  ) -> DispatchResultAs<u32> {
			if class_id == &1u32 {
				return Ok(10u32);
			}
      todo!()
    }

fn get_offer(
    class_id: &pallet_support::NonFungibleClassId,
    offer_id: &u32,
  ) -> pallet_support::DispatchResultAs<(pallet_support::FungibleAssetId, pallet_support::FungibleAssetBalance, pallet_support::AttributeList)> {
		let a1 = Attribute {
			key: br"a1".to_vec().try_into().unwrap(),
			value: AttributeValue::Number(NumberAttribute { number_max: None, number_value: 1})
		};
		let a2 = Attribute {
			key: br"a2".to_vec().try_into().unwrap(),
			value: AttributeValue::String(br"v1".to_vec().try_into().unwrap())
		};
		let attributes: AttributeList = vec![a1.clone(), a2.clone()].try_into().unwrap();	
		if class_id == &1u32 && offer_id == &2u32 {
			return Ok((333, 500, attributes))
		}
		if class_id == &1u32 && offer_id == &3u32 {
			return Ok((1, 10000, attributes))
		}
		Ok((5, 100, attributes))
	}

fn set_attributes(
    class_id: &pallet_support::NonFungibleClassId,
    _asset_id: &pallet_support::NonFungibleAssetId,
    _attributes: pallet_support::AttributeList,
  ) -> frame_support::dispatch::DispatchResult {
		if class_id == &1u32 {
			return Ok(());
		}
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
	type ExecuteOrigin = frame_system::EnsureSigned<u64>;
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
