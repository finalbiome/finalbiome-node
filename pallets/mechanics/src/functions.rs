//! Functions for the Mechnics pallet.
use sp_std::vec::Vec;

use pallet_support::{
  bettor::{Bettor, BettorOutcome, BettorWinning, DrawOutcomeResult, OutcomeResult},
  misc::cumsum_owned,
  traits::{FungibleAssets, NonFungibleAssets},
  DefaultListLengthLimit, DispatchResultAs, LockResultOf, LockedAccet, Locker,
};

use super::*;

impl<T: Config> Pallet<T> {
  /// Gets the id of the mechanic
  pub(crate) fn get_mechanic_id(who: &T::AccountId) -> MechanicIdOf<T> {
    MechanicId::<T::AccountId, T::Index>::from_account_id::<T>(who)
  }

  /// Calculates the block when mechanic should be destroyed
  pub(crate) fn calc_timeout_block() -> BlockNumberFor<T> {
    let block_number = <frame_system::Pallet<T>>::block_number();
    T::MechanicsLifeTime::get().saturating_add(block_number)
  }

  /// Drop mechanic if it exist and clear timeout
  pub(crate) fn drop_mechanic(id: &MechanicIdOf<T>, asset_action: AssetAction) -> DispatchResult {
    let mechanic = Mechanics::<T>::take(&id.account_id, id.nonce);
    if let Some(mechanic) = mechanic {
      Timeouts::<T>::remove(mechanic.get_tiomeout_strorage_key(id.nonce));
      // clear all locks for this mechanic
      for lock in mechanic.locked {
        let origin = Locker::Mechanic(id.clone());
        let who = id.account_id.clone();
        match lock {
          LockedAccet::Nfa(class_id, asset_id) => match asset_action {
            AssetAction::Release => {
              T::NonFungibleAssets::clear_lock(&who, &origin, &class_id, &asset_id)?
            },
            AssetAction::Burn => T::NonFungibleAssets::burn(class_id, asset_id, None)?,
          },
          LockedAccet::Fa(asset_id, amount) => match asset_action {
            AssetAction::Release => todo!("Create an clear_lock method for FA"),
            AssetAction::Burn => {
              let _ = T::FungibleAssets::burn_from(asset_id, &who, amount)?;
            },
          },
        }
      }
    };
    Ok(())
  }

  /// Add lock of asset to the mechanic side
  ///
  /// Don't use it on one's own
  pub(crate) fn try_lock(id: &MechanicIdOf<T>, asset_id: LockedAccet) -> DispatchResult {
    Mechanics::<T>::try_mutate(
      &id.account_id,
      id.nonce,
      |maybe_mechanic| -> DispatchResult {
        match maybe_mechanic {
          Some(ref mut mechanic) => {
            mechanic
              .locked
              .try_push(asset_id)
              .map_err(|_| Error::<T>::AssetsExceedsAllowable)?;
          },
          None => return Err(Error::<T>::MechanicsNotAvailable.into()),
        }
        Ok(())
      },
    )?;
    Ok(())
  }

  /// Clear lock of asset on the mechanic side
  ///
  /// If asset is not found as locked, ignoring it
  /// Don't use it on one's own
  pub(crate) fn _clear_lock(id: &MechanicIdOf<T>, asset_id: LockedAccet) -> DispatchResult {
    Mechanics::<T>::try_mutate(
      &id.account_id,
      id.nonce,
      |maybe_mechanic| -> DispatchResult {
        match maybe_mechanic {
          Some(ref mut mechanic) => {
            if let Some(index) = mechanic.locked.iter().position(|&a| a == asset_id) {
              mechanic.locked.remove(index);
            }
          },
          None => return Err(Error::<T>::MechanicsNotAvailable.into()),
        }
        Ok(())
      },
    )?;
    Ok(())
  }

  /// Try to lock NFA for the given mechanic, both on the mechanic side and asset side.
  pub(crate) fn try_lock_nfa(
    id: &MechanicIdOf<T>,
    who: &AccountIdOf<T>,
    class_id: NonFungibleClassId,
    asset_id: NonFungibleAssetId,
  ) -> DispatchResultAs<LockResultOf<T>> {
    let origin = Locker::Mechanic(id.clone());
    let lock_result = T::NonFungibleAssets::try_lock(who, origin.clone(), &class_id, &asset_id)?;
    Self::try_lock(id, LockedAccet::Nfa(class_id, asset_id)).map_err(|e| {
      // rollback if something goes wrong
      let _ = T::NonFungibleAssets::clear_lock(who, &origin, &class_id, &asset_id);
      e
    })?;
    Ok(lock_result)
  }

  /// Clear lock for NFA
  ///
  /// Any error will be suppressed
  pub(crate) fn _clear_lock_nfa(
    id: &MechanicIdOf<T>,
    who: &AccountIdOf<T>,
    class_id: NonFungibleClassId,
    asset_id: NonFungibleAssetId,
  ) -> DispatchResult {
    let origin = Locker::Mechanic(id.clone());
    let _ = T::NonFungibleAssets::clear_lock(who, &origin, &class_id, &asset_id);
    let res = Self::_clear_lock(id, LockedAccet::Nfa(class_id, asset_id));
    debug_assert!(res.is_ok(), "clear_lock rise error, but shouldn't");
    Ok(())
  }

  /// Execute Mechanic `exec_buy_nfa`
  pub(crate) fn do_buy_nfa(
    who: &T::AccountId,
    class_id: &NonFungibleClassId,
    offer_id: &u32,
  ) -> DispatchResultAs<NonFungibleAssetId> {
    // checking availability of that mechanic for the nfa class
    let (fa, price, attributes) = T::NonFungibleAssets::get_offer(class_id, offer_id)?;
    // check fa balances
    T::FungibleAssets::can_withdraw(fa, who, price).into_result()?;
    // mint nfa
    let asset_id = T::NonFungibleAssets::mint_into(class_id, who)?;
    // set attributes
    T::NonFungibleAssets::set_attributes(&asset_id, attributes)?;
    // withdraw
    T::FungibleAssets::burn_from(fa, who, price)?;
    Ok(asset_id)
  }

  /// Process a bet mechanic the first time. \
  /// Here check given asset to acceptance for the Bet mechanic,
  /// Create new mechanic and execute first round.
  pub(crate) fn do_bet(
    who: &T::AccountId,
    class_id: &NonFungibleClassId,
    asset_id: &NonFungibleAssetId,
  ) -> DispatchResult {
    let mechanic_id = Self::get_mechanic_id(who);

    // create the mechanic data
    let data = MechanicData::Bet(MechanicDataBet::default());
    let mechanic = MechanicDetailsBuilder::build::<T>(mechanic_id.account_id.clone(), data);
    let timeout_key = mechanic.get_tiomeout_strorage_key(mechanic_id.nonce);
    Mechanics::<T>::insert(&mechanic_id.account_id, mechanic_id.nonce, mechanic);
    Timeouts::<T>::insert(timeout_key, ());

    let _ = Self::try_lock_nfa(&mechanic_id, who, *class_id, *asset_id).map_err(|err| {
      let _ = Self::drop_mechanic(&mechanic_id, AssetAction::Release);
      err
    })?;
    let class_details = T::NonFungibleAssets::get_class(class_id)?;
    // we need to check can it acceptable to this mechanic
    Self::can_use_mechanic(&Mechanic::Bet, &class_details).map_err(|err| {
      // clean the lock
      let _ = Self::drop_mechanic(&mechanic_id, AssetAction::Release);
      err
    })?;
    if let Some(bettor) = class_details.bettor {
      // no results exist for the first time played asset
      let outcomes = Vec::new();

      Self::play_bet_round(who, mechanic_id, &bettor, outcomes)?;
    };
    Ok(())
  }

  /// Process existed Bet mechanic by it id. \
  /// Where is no checks about the bettor asset -
  /// any checks has been executed in [do_bet()]
  pub(crate) fn do_bet_next_round(
    who: &T::AccountId,
    mechanic_id: MechanicIdOf<T>,
  ) -> DispatchResult {
    // check validity of id
    mechanic_id
      .ensure_owner(who)
      .map_err(|_| Error::<T>::MechanicsNotAvailable)?;
    let mechanic = Mechanics::<T>::try_get(&mechanic_id.account_id, mechanic_id.nonce)
      .map_err(|_| Error::<T>::MechanicsNotAvailable)?;
    // get bet asset from mechanic lock
    // for the Bet mechanic only one asset can be using
    if mechanic.locked.len() != 1 {
      debug_assert!(
        false,
        "for the Bet mechanic only one asset can be using (locked)"
      );
      return Err(Error::<T>::Internal.into());
    }
    let bet_asset = mechanic.locked[0];
    let (class_id, _) = match bet_asset {
      LockedAccet::Nfa(class_id, asset_id) => (class_id, asset_id),
      // expect only NFA Bettor
      LockedAccet::Fa(_, _) => return Err(Error::<T>::Internal.into()),
    };
    let class_details = T::NonFungibleAssets::get_class(&class_id)?;

    // get played outcomes
    let outcomes = if let MechanicData::Bet(data_bet) = mechanic.data {
      data_bet.outcomes.into_inner()
    } else {
      debug_assert!(
        false,
        "a mechanic data whitout the bettor can't be present here"
      );
      Vec::new()
    };
    if let Some(bettor) = class_details.bettor {
      Self::play_bet_round(who, mechanic_id, &bettor, outcomes)?;
    } else {
      debug_assert!(false, "an asset whitout the bettor can't be present here");
      return Err(Error::<T>::Internal.into());
    }
    Ok(())
  }

  /// Play any round for the Bet mechanic.  \
  /// `outcomes` represent an already played rounds
  pub(crate) fn play_bet_round(
    who: &T::AccountId,
    mechanic_id: MechanicIdOf<T>,
    bettor: &Bettor,
    outcomes: Vec<u32>,
  ) -> DispatchResult {
    // play round
    let result: u32 = Self::choose_outcome(&mechanic_id, &bettor.outcomes);
    let mut outcomes = outcomes;
    outcomes.push(result);

    let played_rounds = outcomes.len();
    // trying to determine the final result
    if let Some(bet_result) = Self::try_finalize_bet(&outcomes, bettor.rounds, &bettor.outcomes) {
      // finish mechanic
      Self::do_bet_result_processing(&mechanic_id, who, bettor, bet_result, outcomes)?;
    } else if played_rounds == bettor.rounds as usize {
      // the bet was completed, but the winner was not found, i.e. draw
      Self::do_bet_result_processing(&mechanic_id, who, bettor, BetResult::Draw, outcomes)?;
    } else {
      debug_assert!(bettor.rounds > 1, "bettor cannot have one round here");
      // save results to mechanic data for future uses
      Self::add_bet_result(&mechanic_id, &outcomes)?;
      Self::deposit_event(Event::Stopped {
        id: mechanic_id.nonce,
        owner: mechanic_id.account_id,
        reason: EventMechanicStopReason::UpgradeNeeded,
      });
    };
    Ok(())
  }

  /// Generate a random number from a given seed.
  /// Note that there is potential bias introduced by using modulus operator.
  /// You should call this function with different seed values until the random
  /// number lies within `u32::MAX - u32::MAX % n`.
  /// TODO: deal with randomness freshness
  /// https://github.com/paritytech/substrate/issues/8311
  /// Taken from https://github.com/paritytech/substrate/blob/d602397a0bbb24b5d627795b797259a44a5e29e9/frame/lottery/src/lib.rs#L506
  fn generate_random_number(mechanic_id: &MechanicIdOf<T>, seed_suffix: u32) -> u32 {
    let (random_seed, _) = T::Randomness::random(&(mechanic_id, seed_suffix).encode());
    let random_number = <u32>::decode(&mut random_seed.as_ref())
      .expect("secure hashes should always be bigger than u32; qed");
    random_number
  }

  /// Randomly choose a variant from among the total number of variants (`(1..total)`)
  pub(crate) fn choose_variant(mechanic_id: &MechanicIdOf<T>, total: u32) -> u32 {
    let mut random_number = Self::generate_random_number(mechanic_id, 0);
    // Best effort attempt to remove bias from modulus operator.
    for i in 1..total {
      if random_number < u32::MAX - u32::MAX % total {
        break;
      }
      random_number = Self::generate_random_number(mechanic_id, i);
    }
    random_number % total
  }

  /// Randomly choose an outcome index by given probability
  ///
  /// Returns an index of some selected outcomes.  \
  /// **Warn**: `outcomes` must be from the valid [Bettor]
  pub(crate) fn choose_outcome(
    mechanic_id: &MechanicIdOf<T>,
    outcomes: &BoundedVec<BettorOutcome, DefaultListLengthLimit>,
  ) -> u32 {
    let probs = cumsum_owned(outcomes.iter().map(|o| o.probability).collect());
    let random_variant = Self::choose_variant(mechanic_id, probs[probs.len() - 1]);
    let mut chosen_outcome_idx = probs.len() - 1;
    for (idx, prob) in probs.into_iter().enumerate() {
      if prob > random_variant {
        chosen_outcome_idx = idx;
        break;
      }
    }
    chosen_outcome_idx
      .try_into()
      .expect("BoundedVec index can't overfolow DefaultListLengthLimit")
  }

  /// Trying to determine the winner by completed rounds
  ///
  /// If win or loss cannot be determined, None is returned.
  pub(crate) fn try_finalize_bet(
    completed: &[u32],
    total_rounds: u32,
    outcomes: &BoundedVec<BettorOutcome, DefaultListLengthLimit>,
  ) -> Option<BetResult> {
    let mut won_rounds = 0u32;
    let mut lost_rounds = 0u32;
    let mut draw_rounds = 0u32;
    for round_result_idx in completed {
      let round_result = &outcomes[*round_result_idx as usize];
      match round_result.result {
        OutcomeResult::Win => won_rounds += 1,
        OutcomeResult::Lose => lost_rounds += 1,
        OutcomeResult::Draw => draw_rounds += 1,
      }
    }

    if total_rounds
      .saturating_sub(draw_rounds)
      .saturating_sub(lost_rounds)
      < lost_rounds
    {
      Some(BetResult::Lost)
    } else if total_rounds
      .saturating_sub(draw_rounds)
      .saturating_sub(won_rounds)
      < won_rounds
    {
      Some(BetResult::Won)
    } else {
      None
    }
  }

  /// Processing with assets by Bet mechanic results
  ///
  /// Mint assets if needed, drop mechanic, emit events
  pub(crate) fn do_bet_result_processing(
    mechanic_id: &MechanicIdOf<T>,
    who: &T::AccountId,
    bettor: &Bettor,
    result: BetResult,
    outcomes: Vec<u32>,
  ) -> DispatchResult {
    let result = match result {
      BetResult::Won => BetResult::Won,
      BetResult::Lost => BetResult::Lost,
      BetResult::Draw => match bettor.draw_outcome {
        DrawOutcomeResult::Win => BetResult::Won,
        DrawOutcomeResult::Lose => BetResult::Lost,
        DrawOutcomeResult::Keep => BetResult::Draw,
      },
    };

    match result {
      BetResult::Won => {
        // mint assets
        for wining in bettor.winnings.clone() {
          match wining {
            BettorWinning::Fa(asset_id, amount) => {
              T::FungibleAssets::mint_into(asset_id, who, amount)?;
            },
            BettorWinning::Nfa(class_id) => {
              T::NonFungibleAssets::mint_into(&class_id, who)?;
            },
          }
        }
        // drop mechanic with burning bettor asset
        Self::drop_mechanic(mechanic_id, AssetAction::Burn)?;
      },
      BetResult::Lost => {
        // drop mechanic with burning bettor asset
        Self::drop_mechanic(mechanic_id, AssetAction::Burn)?;
      },
      BetResult::Draw => {
        // drop mechanic but not burning bettor asset
        Self::drop_mechanic(mechanic_id, AssetAction::Release)?;
      },
    };
    // drop mechanic
    // emit Finished event
    let outcomes: MechanicDataBetOutcomes = outcomes
      .to_vec()
      .try_into()
      .expect("the number of values cannot exceed the number of rounds");
    let result: EventMechanicResult =
      Some(EventMechanicResultData::Bet(EventMechanicResultDataBet {
        outcomes,
        result,
      }));
    Self::deposit_event(Event::Finished {
      id: mechanic_id.nonce,
      owner: mechanic_id.account_id.clone(),
      result,
    });
    Ok(())
  }

  /// Add intermediate result to the Bet mechanic data
  pub(crate) fn add_bet_result(id: &MechanicIdOf<T>, outcomes: &[u32]) -> DispatchResult {
    Mechanics::<T>::try_mutate(
      &id.account_id,
      id.nonce,
      |maybe_mechanic| -> DispatchResult {
        let outcomes: MechanicDataBetOutcomes = outcomes
          .to_vec()
          .try_into()
          .expect("the number of values cannot exceed the number of rounds");
        let data = MechanicData::Bet(MechanicDataBet { outcomes });
        match maybe_mechanic {
          Some(ref mut mechanic) => {
            mechanic.data = data;
          },
          maybe_mechanic @ None => {
            let mechanic = MechanicDetailsBuilder::build::<T>(id.account_id.clone(), data);
            Timeouts::<T>::insert(mechanic.get_tiomeout_strorage_key(id.nonce), ());
            *maybe_mechanic = Some(mechanic);
          },
        }
        Ok(())
      },
    )?;
    Ok(())
  }

  /// Checks if a class can be used for a given mechanic
  pub(crate) fn can_use_mechanic(
    mechanic: &Mechanic,
    class_details: &ClassDetailsOf<T>,
  ) -> DispatchResult {
    match mechanic {
      Mechanic::BuyNfa => {
        if class_details.purchased.is_none() {
          Err(Error::<T>::IncompatibleAsset.into())
        } else {
          Ok(())
        }
      },
      Mechanic::Bet => {
        if class_details.bettor.is_none() {
          Err(Error::<T>::IncompatibleAsset.into())
        } else {
          Ok(())
        }
      },
    }
  }

  /// Upgrage mechanic by given data and try to execute it.
  pub(crate) fn do_upgrade(
    who: &AccountIdOf<T>,
    upgrage_data: MechanicUpgradeDataOf<T>,
  ) -> DispatchResult {
    // check validity of id
    upgrage_data
      .mechanic_id
      .ensure_owner(who)
      .map_err(|_| Error::<T>::NoPermission)?;
    // checks an mechanic existance
    let mechanic = Mechanics::<T>::try_get(
      &upgrage_data.mechanic_id.account_id,
      upgrage_data.mechanic_id.nonce,
    )
    .map_err(|_| Error::<T>::MechanicsNotAvailable)?;
    // checks mechanic owner
    ensure!(&mechanic.owner == who, Error::<T>::NoPermission);

    // validate data
    // ensure compatibility
    ensure!(
      Mechanic::from(&upgrage_data.payload) == Mechanic::from(&mechanic.data),
      Error::<T>::IncompatibleData
    );
    // upgrade mechanic / update mechanic data
    // execute mechanic

    match upgrage_data.payload {
      MechanicUpgradePayload::Bet => {
        // for bet mechanic just execute next round
        Self::do_bet_next_round(who, upgrage_data.mechanic_id)?;
      },
    }

    Ok(())
  }

  /// Drop all mechanics by timeout.
  ///
  /// Returns weight and count of dropped mechanics
  pub(crate) fn process_mechanic_timeouts() -> (Weight, u32) {
    let mut mechanics_count = 0;
    let curr_block = <frame_system::Pallet<T>>::block_number();
    for (mechanic_id, _) in Timeouts::<T>::drain_prefix((curr_block,)) {
      let id = MechanicId {
        account_id: mechanic_id.0,
        nonce: mechanic_id.1,
      };
      let _ = Self::drop_mechanic(&id, AssetAction::Burn);
      mechanics_count = mechanics_count.saturating_add(1);
    }
    (0, mechanics_count)
  }
}
