use crate as pallet_mechanics;
use codec::Encode;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, Hooks};
use frame_system as system;
use pallet_support::{
  bettor::{Bettor, BettorOutcome, BettorWinning, DrawOutcomeResult, OutcomeResult},
  types_nfa::{AssetDetails, ClassDetails},
  *,
};
use sp_core::H256;
use sp_runtime::{
  testing::Header,
  traits::{BlakeTwo256, IdentityLookup},
};
// use frame_support_test::TestRandomness;

use super::bvec;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type BlockNumber = u64;

/// Provides an implementation of [frame_support::traits::Randomness]
/// that should only be used in tests!
/// TestRandomness returns a block number as ramdom value
pub struct TestRandomness<T>(sp_std::marker::PhantomData<T>);
impl<Output: codec::Decode + Default, T> frame_support::traits::Randomness<Output, T::BlockNumber>
  for TestRandomness<T>
where
  T: frame_system::Config,
{
  fn random(_subject: &[u8]) -> (Output, T::BlockNumber) {
    use sp_runtime::traits::TrailingZeroInput;
    let block_number: u32 = frame_system::Pallet::<T>::block_number()
      .try_into()
      .unwrap_or_default();
    (
      Output::decode(&mut TrailingZeroInput::new(&block_number.encode())).unwrap_or_default(),
      frame_system::Pallet::<T>::block_number(),
    )
  }
}

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

/// Mock of fungible-assets-pallet impl
pub struct FAPallet {}
impl pallet_support::traits::FungibleAssets<u64> for FAPallet {
  fn can_withdraw(
    asset: FungibleAssetId,
    _who: &u64,
    amount: FungibleAssetBalance,
  ) -> frame_support::traits::tokens::WithdrawConsequence<FungibleAssetBalance> {
    if amount > 9999.into() {
      return frame_support::traits::tokens::WithdrawConsequence::NoFunds;
    }
    if asset == 333.into() {
      return frame_support::traits::tokens::WithdrawConsequence::UnknownAsset;
    }
    frame_support::traits::tokens::WithdrawConsequence::Success
  }

  fn burn_from(
    asset_id: FungibleAssetId,
    _who: &u64,
    _amount: FungibleAssetBalance,
  ) -> pallet_support::DispatchResultAs<FungibleAssetBalance> {
    if asset_id == 5u32.into() {
      return Ok(10000.into());
    }
    todo!()
  }

  fn inc_references(_asset: &FungibleAssetId) -> sp_runtime::DispatchResult {
    Ok(())
  }

  fn dec_references(_asset: &FungibleAssetId) -> sp_runtime::DispatchResult {
    Ok(())
  }

  fn mint_into(
    _asset: FungibleAssetId,
    _who: &u64,
    _amount: FungibleAssetBalance,
  ) -> sp_runtime::DispatchResult {
    todo!()
  }
}

/// Mock of non-fungible-assets-pallet impl
pub struct NFAPallet {}
impl pallet_support::traits::NonFungibleAssets<u64, u32> for NFAPallet {
  fn mint_into(class_id: &NonFungibleClassId, _who: &u64) -> DispatchResultAs<NonFungibleAssetId> {
    if class_id == &1u32.into() {
      return Ok(10u32.into());
    }
    if class_id == &10u32.into() {
      // test do_bet_result_processing_win_nfa
      return Ok(11u32.into());
    }
    if class_id == &11u32.into() {
      // test do_bet_result_processing_draw_win
      return Ok(11u32.into());
    }
    if class_id == &14u32.into() {
      // test play_bet_round_single_round_win
      return Ok(11u32.into());
    }
    if class_id == &16u32.into() {
      // test play_bet_round_three_rounds_win_at_second_round
      return Ok(11u32.into());
    }
    if class_id == &19u32.into() {
      // test play_bet_round_single_round_draw_win
      return Ok(11u32.into());
    }
    if class_id == &20u32.into() {
      // test do_bet_asset_one_round_work
      return Ok(11u32.into());
    }
    todo!()
  }

  fn get_offer(
    class_id: &pallet_support::NonFungibleClassId,
    offer_id: &u32,
  ) -> pallet_support::DispatchResultAs<(
    pallet_support::FungibleAssetId,
    pallet_support::FungibleAssetBalance,
    pallet_support::AttributeList,
  )> {
    let a1 = Attribute {
      key: br"a1".to_vec().try_into().unwrap(),
      value: AttributeValue::Number(NumberAttribute {
        number_max: None,
        number_value: 1,
      }),
    };
    let a2 = Attribute {
      key: br"a2".to_vec().try_into().unwrap(),
      value: AttributeValue::Text(br"v1".to_vec().try_into().unwrap()),
    };
    let attributes: AttributeList = vec![a1, a2].try_into().unwrap();
    if class_id == &1u32.into() && offer_id == &2u32 {
      return Ok((333.into(), 500.into(), attributes));
    }
    if class_id == &1u32.into() && offer_id == &3u32 {
      return Ok((1.into(), 10000.into(), attributes));
    }
    Ok((5.into(), 100.into(), attributes))
  }

  fn set_attributes(
    asset_id: &pallet_support::NonFungibleAssetId,
    _attributes: pallet_support::AttributeList,
  ) -> frame_support::dispatch::DispatchResult {
    if asset_id == &10u32.into() {
      return Ok(());
    }
    todo!()
  }
  fn try_lock(
    who: &u64,
    origin: Locker<u64, u32>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> DispatchResultAs<LockResult<u64, u32>> {
    if class_id == &2u32.into() && asset_id == &3u32.into() {
      // test try_lock_nfa_works
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &4u32.into() && asset_id == &5u32.into() {
      // test crear_lock_nfa_works
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &5u32.into() && asset_id == &6u32.into() {
      // test do_bet_unexisted_asset
      return Err(sp_runtime::DispatchError::Other(
        "mock_error_asset_doesnt_exist",
      ));
    }
    if class_id == &6u32.into() && asset_id == &7u32.into() {
      // test crear_lock_nfa_works
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &6u32.into() && asset_id == &8u32.into() {
      // test do_bet_unexisted_mechanic
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &7u32.into() && asset_id == &7u32.into() {
      // test do_bet_asset_not_bettor
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &34u32.into() && asset_id == &34u32.into() {
      // test do_bet_asset_one_round_work
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &35u32.into() && asset_id == &35u32.into() {
      // test do_bet_next_round_two_rounds_work
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    if class_id == &36u32.into() && asset_id == &36u32.into() {
      // test do_do_upgrade_bet_two_rounds_work
      let lr: LockResultOf<Test> = LockResult::Locked(AssetDetails {
        locked: origin,
        owner: *who,
      });
      return Ok(lr);
    }
    todo!()
  }
  fn get_class(class_id: &NonFungibleClassId) -> DispatchResultAs<ClassDetails<u64>> {
    if class_id == &6.into() {
      // test do_bet_unexisted_mechanic, do_bet_unexisted_mechanic
      return Ok(ClassDetails {
        attributes: 0,
        name: br"a".to_vec().try_into().unwrap(),
        bettor: None,
        purchased: None,
        instances: 0,
        owner: 1,
      });
    }
    if class_id == &7.into() {
      // test do_bet_asset_not_bettor
      return Ok(ClassDetails {
        attributes: 0,
        name: br"a".to_vec().try_into().unwrap(),
        bettor: None,
        purchased: None,
        instances: 0,
        owner: 1,
      });
    }
    if class_id == &34.into() {
      // test do_bet_asset_one_round_work
      return Ok(ClassDetails {
        attributes: 0,
        name: br"a".to_vec().try_into().unwrap(),
        bettor: Some(Bettor {
          // with odd block numbers the bettor will lose
          // with even - win
          outcomes: bvec![
            BettorOutcome {
              name: bvec!(br"o1"),
              probability: 1,
              result: OutcomeResult::Win,
            },
            BettorOutcome {
              name: bvec!(br"o2"),
              probability: 1,
              result: OutcomeResult::Lose,
            },
          ],
          winnings: bvec![BettorWinning::Nfa(20.into())],
          rounds: 1,
          draw_outcome: DrawOutcomeResult::Keep,
        }),
        purchased: None,
        instances: 0,
        owner: 1,
      });
    }
    if class_id == &35.into() {
      // test do_bet_next_round_two_rounds_work
      return Ok(ClassDetails {
        attributes: 0,
        name: br"a".to_vec().try_into().unwrap(),
        bettor: Some(Bettor {
          // with odd block numbers the bettor will lose
          // with even - win
          outcomes: bvec![
            BettorOutcome {
              name: bvec!(br"o1"),
              probability: 1,
              result: OutcomeResult::Win,
            },
            BettorOutcome {
              name: bvec!(br"o2"),
              probability: 1,
              result: OutcomeResult::Lose,
            },
          ],
          winnings: bvec![BettorWinning::Nfa(20.into())],
          rounds: 2,
          draw_outcome: DrawOutcomeResult::Lose,
        }),
        purchased: None,
        instances: 0,
        owner: 1,
      });
    }
    if class_id == &36.into() {
      // test do_do_upgrade_bet_two_rounds_work
      return Ok(ClassDetails {
        attributes: 0,
        name: br"a".to_vec().try_into().unwrap(),
        bettor: Some(Bettor {
          // with odd block numbers the bettor will lose
          // with even - win
          outcomes: bvec![
            BettorOutcome {
              name: bvec!(br"o1"),
              probability: 1,
              result: OutcomeResult::Win,
            },
            BettorOutcome {
              name: bvec!(br"o2"),
              probability: 1,
              result: OutcomeResult::Lose,
            },
          ],
          winnings: bvec![BettorWinning::Nfa(20.into())],
          rounds: 2,
          draw_outcome: DrawOutcomeResult::Lose,
        }),
        purchased: None,
        instances: 0,
        owner: 1,
      });
    }
    todo!()
  }
  fn burn(
    class_id: NonFungibleClassId,
    asset_id: NonFungibleAssetId,
    _maybe_check_owner: Option<&u64>,
  ) -> sp_runtime::DispatchResult {
    if class_id == 22u32.into() && asset_id == 33u32.into() {
      // test do_bet_result_processing_win_nfa
      return Ok(());
    }
    if class_id == 23u32.into() && asset_id == 34u32.into() {
      // test do_bet_result_processing_lose
      return Ok(());
    }
    if class_id == 24u32.into() && asset_id == 35u32.into() {
      // test do_bet_result_processing_draw_win
      return Ok(());
    }
    if class_id == 25u32.into() && asset_id == 36u32.into() {
      // test do_bet_result_processing_draw_lose
      return Ok(());
    }
    if class_id == 8u32.into() && asset_id == 8u32.into() {
      // test play_bet_round_single_round_win
      return Ok(());
    }
    if class_id == 9u32.into() && asset_id == 9u32.into() {
      // test play_bet_round_single_round_lose
      return Ok(());
    }
    if class_id == 30u32.into() && asset_id == 30u32.into() {
      // test play_bet_round_three_rounds_win_at_second_round
      return Ok(());
    }
    if class_id == 32u32.into() && asset_id == 32u32.into() {
      // test play_bet_round_single_round_draw_lose
      return Ok(());
    }
    if class_id == 33u32.into() && asset_id == 33u32.into() {
      // test play_bet_round_single_round_draw_win
      return Ok(());
    }
    if class_id == 34u32.into() && asset_id == 34u32.into() {
      // test do_bet_asset_one_round_work
      return Ok(());
    }
    if class_id == 35u32.into() && asset_id == 35u32.into() {
      // test do_bet_next_round_two_rounds_work
      return Ok(());
    }
    if class_id == 36u32.into() && asset_id == 36u32.into() {
      // test do_do_upgrade_bet_two_rounds_work
      return Ok(());
    }
    todo!()
  }
  fn clear_lock(
    _who: &u64,
    _origin: &Locker<u64, u32>,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> sp_runtime::DispatchResult {
    if class_id == &4.into() && asset_id == &5.into() {
      return Ok(());
    }
    if class_id == &6.into() && asset_id == &8.into() {
      // test do_bet_unexisted_mechanic
      return Ok(());
    }
    if class_id == &7.into() && asset_id == &7.into() {
      // test do_bet_asset_not_bettor
      return Ok(());
    }
    if class_id == &34.into() && asset_id == &34.into() {
      // test do_bet_asset_one_round_work
      return Ok(());
    }
    if class_id == &35.into() && asset_id == &35.into() {
      // test do_bet_next_round_two_rounds_work
      return Ok(());
    }
    if class_id == &36.into() && asset_id == &36.into() {
      // test do_do_upgrade_bet_two_rounds_work
      return Ok(());
    }
    todo!()
  }
}

impl pallet_mechanics::Config for Test {
  type Event = Event;
  type FungibleAssets = FAPallet;
  type NonFungibleAssets = NFAPallet;
  type Randomness = TestRandomness<Self>;
  type AssetsListLimit = ConstU32<16>;
  type MechanicsLifeTime = ConstU64<20>;
  type ExecuteOrigin = frame_system::EnsureSigned<u64>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let storage = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();
  let mut ext: sp_io::TestExternalities = storage.into();
  ext.execute_with(|| {
    System::set_block_number(1);
    System::on_initialize(1);
    MechanicsModule::on_initialize(1);
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
    MechanicsModule::on_finalize(System::block_number());
    System::on_finalize(System::block_number());

    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    MechanicsModule::on_initialize(System::block_number());
  }
}
