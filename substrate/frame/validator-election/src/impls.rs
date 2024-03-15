use frame_system::pallet_prelude::BlockNumberFor;

use crate::*;

pub trait CollectiveInterface<AccountId> {
	fn set_new_members(new: Vec<AccountId>);
}

impl<AccountId> CollectiveInterface<AccountId> for () {
	fn set_new_members(_new: Vec<AccountId>) {}
}

pub trait SessionAlert<BlockNumber> {
	/// Whether new session has triggered
	fn is_new_session(n: BlockNumber) -> bool;
}

impl<T: Config> SessionAlert<BlockNumberFor<T>> for Pallet<T> {
	fn is_new_session(_n: BlockNumberFor<T>) -> bool {
		true
	}
}

/// Something that handles fee reward
pub trait RewardInterface<Location, Balance> {
	/// Fee will be aggregated on certain account for current session
	fn aggregate_reward(session_index: SessionIndex, system_token_id: Location, amount: Balance);
	/// Fee will be distributed to the validators for current session
	fn distribute_reward(session_index: SessionIndex);
}

impl<Location, Balance> RewardInterface<Location, Balance> for () {
	fn aggregate_reward(
		_session_index: SessionIndex,
		_system_token_id: Location,
		_amount: Balance,
	) {
	}
	fn distribute_reward(_session_index: SessionIndex) {}
}

impl<T: Config> TaaV for Pallet<T> {
	type Error = sp_runtime::DispatchError;

	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
		// Try decode
		let vote = PotVote::<T::AccountId, T::Score>::decode(&mut &bytes[..]).map_err(|_| Error::<T>::ErrorDecode)?;
		log::info!("🥶🥶 Processing Vote: {:?}", vote);
		let PotVote { candidate, weight } = vote;
		if SeedTrustValidatorPool::<T>::get().contains(&candidate) {
			return Ok(())
		}
		// @Hugo
		// let block_time_weight: F64 = {
		// 	let current_block_number: u128 = relay_parent_number.saturated_into();
		// 	// pow = ln(2) * current block number / BLOCKS_PER_YEAR
		// 	let pow: F64 = F64::from_i128(2).ln() *
		// 		F64::from_i128(current_block_number as i128).div(BLOCKS_PER_YEAR);
		// 	// block_time_weight = 2 ^ (current block number / BLOCKS_PER_YEAR) = exp ^ (pow)
		// 	let block_time_weight = pow.exp();
		// 	block_time_weight
		// };
		PotValidatorPool::<T>::mutate(|voting_status| {
			voting_status.add_vote(&candidate, weight.clone());
		});
		// @Sirius
		// T::RewardInterface::aggregate_reward (
		// 	session_index,
		// 	system_token_id.para_id, -> DestId in RewardInterface
		// 	original, -> `AssetId` in PotVote
		// 	adjusted_weight,
		// );
		Self::deposit_event(Event::<T>::Voted { who: candidate, amount: weight });
		Ok(())
	}
}

/// Means for interacting with a specialized version of the `session` trait.
pub trait SessionInterface<AccountId> {
	/// Disable the validator at the given index, returns `false` if the validator was already
	/// disabled or the index is out of bounds.
	fn disable_validator(validator_index: u32) -> bool;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Config> SessionInterface<<T as frame_system::Config>::AccountId> for T
where
	T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
{
	fn disable_validator(validator_index: u32) -> bool {
		pallet_session::Pallet::<T>::disable_index(validator_index)
	}

	fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
		pallet_session::Pallet::<T>::validators()
	}

	fn prune_historical_up_to(_up_to: SessionIndex) {
		()
	}
}

impl<AccountId> SessionInterface<AccountId> for () {
	fn disable_validator(_: u32) -> bool {
		true
	}
	fn validators() -> Vec<AccountId> {
		Vec::new()
	}
	fn prune_historical_up_to(_: SessionIndex) {
		()
	}
}

/// Interface for Proof-of-Transaction
pub trait PotInterface<Account> {
	/// Actual weight type for `proof-of-transaction`
	type VoteWeight: frame_support::Parameter + Into<F64>;

	/// Update the vote status for the given account.
	fn vote(who: Account, weight: Self::VoteWeight) -> bool;
}

impl<T: Config, Account: Clone> PotInterface<Account> for Pallet<T>
where
	T::AccountId: From<Account>,
{
	type VoteWeight = T::Score;

	fn vote(who: Account, weight: Self::VoteWeight) -> bool {
		// Return if vote candidate is in SeedTrustValidatorPool
		if SeedTrustValidatorPool::<T>::get().contains(&who.clone().into()) {
			return false
		}
		PotValidatorPool::<T>::mutate(|voting_status| {
			voting_status.add_vote(&who.into(), weight);
		});
		true
	}
}

impl<Account> PotInterface<Account> for () {
	type VoteWeight = F64;

	fn vote(_: Account, _: Self::VoteWeight) -> bool {
		false
	}
}

// Session Pallet Rotate Order
//
// On Genesis:
// `new_session_genesis()` is called
//
// After Genesis:
// `on_initialize(block_number)` when session is about to end
// `end_session(bn)` -> `start_session(bn+1)` -> `new_session(bn+2)` are called this order
//
// Detail
// 1. new_session()
// - Plan a new session and optionally returns Validator Set
// - Potentially plan a new era
// - Internally `trigger_new_era()` is called when planning a new era
//
// 2. new_session_genesis()
// - Called only once at genesis
// - If there is no validator set returned, session pallet's config keys are used for initial
//   validator set
//
// 3. start_session()
// - Start a session potentially starting an era
// - Internally `start_era()` is called when starting a new era
//
// 4. end_session()
// - End a session potentially ending an era
// - Internally `end_era()` is called when ending an era
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::handle_new_session(new_index, false)
	}
	fn new_session_genesis(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::handle_new_session(new_index, true)
	}
	fn start_session(start_index: SessionIndex) {
		log!(info, "⏰ starting session {}", start_index);
	}
	fn end_session(end_index: SessionIndex) {
		log!(info, "⏰ ending session {}", end_index);
		// T::RewardInterface::distribute_reward(end_index);
	}
}

impl<T: Config> Pallet<T> {
	fn handle_new_session(
		session_index: SessionIndex,
		is_genesis: bool,
	) -> Option<Vec<T::AccountId>> {
		if let Some(current_era) = CurrentEra::<T>::get() {
			let start_session_index = StartSessionIndexPerEra::<T>::get(current_era)
				.unwrap_or_else(|| {
					frame_support::print("Error: start_session_index must be set for current_era");
					0
				});
			let era_length = session_index.saturating_sub(start_session_index); // Must never happen.

			match ForceEra::<T>::get() {
				// Will be set to `NotForcing` again if a new era has been triggered.
				Forcing::ForceNew => (),
				// Short circuit to `try_trigger_new_era`.
				Forcing::ForceAlways => (),
				// Only go to `try_trigger_new_era` if deadline reached.
				Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
				_ => {
					// Either `Forcing::ForceNone`,
					// or `Forcing::NotForcing if era_length < T::SessionsPerEra::get()`.
					return None
				},
			}

			// New era.
			let maybe_new_era_validators = Self::do_trigger_new_era(session_index, is_genesis);
			if maybe_new_era_validators.is_some() &&
				matches!(ForceEra::<T>::get(), Forcing::ForceNew)
			{
				Self::set_force_era(Forcing::NotForcing);
			}

			maybe_new_era_validators
		} else {
			log!(debug, "Starting the first era.");
			Self::do_trigger_new_era(session_index, is_genesis)
		}
	}

	/// Plan a new era.
	///
	/// * Bump the current era storage (which holds the latest planned era).
	/// * Store start session index for the new planned era.
	/// * Clean old era information.
	/// * Store staking information for the new planned era
	///
	/// Returns the new validator set.
	fn do_trigger_new_era(
		session_index: SessionIndex,
		_is_genesis: bool,
	) -> Option<Vec<T::AccountId>> {
		let new_planned_era = CurrentEra::<T>::mutate(|era| {
			*era = Some(era.map(|old_era| old_era + 1).unwrap_or(0));
			era.unwrap()
		});
		StartSessionIndexPerEra::<T>::insert(&new_planned_era, session_index);
		Self::deposit_event(Event::<T>::NewEraTriggered { era_index: new_planned_era });
		Some(Self::elect_validators(new_planned_era))

		// Clean old era information.
		// Later
		// if let Some(old_era) = new_planned_era.checked_sub(T::HistoryDepth::get() + 1) {
		// 	Self::clear_era_information(old_era);
		// }
	}

	/// Elect validators from `SeedTrustValidatorPool::<T>` and `PotValidatorPool::<T>`
	///
	/// First, check the number of seed trust validator.
	/// If it is equal to number of max validators, we just elect from
	/// `SeedTrustValidatorPool::<T>`. Otherwise, remain number of validators are elected from
	/// `PotValidatorPool::<T>`.
	pub fn elect_validators(era_index: EraIndex) -> Vec<T::AccountId> {
		let total_num_validators = TotalValidatorSlots::<T>::get();
		let seed_trust_slots = SeedTrustSlots::<T>::get();
		let num_pot = total_num_validators - seed_trust_slots;
		let mut pot_enabled = false;
		let mut maybe_new_validators: Vec<T::AccountId> =
			Self::do_elect_seed_trust_validators(seed_trust_slots);
		if Self::is_pot_enabled(num_pot) {
			let mut pot_validators = Self::do_elect_pot_validators(era_index, num_pot);
			pot_enabled = true;
			maybe_new_validators.append(&mut pot_validators);
		}
		let old_validators = T::SessionInterface::validators();
		if old_validators == maybe_new_validators {
			Self::deposit_event(Event::<T>::ValidatorsNotChanged);
			return old_validators
		}
		Self::deposit_event(Event::<T>::ValidatorsElected {
			validators: maybe_new_validators.clone(),
			pot_enabled,
		});
		T::CollectiveInterface::set_new_members(maybe_new_validators.clone());
		maybe_new_validators
	}

	fn do_elect_seed_trust_validators(seed_trust_slots: u32) -> Vec<T::AccountId> {
		log!(trace, "Elect seed trust validators");
		let seed_trust_validators = SeedTrustValidatorPool::<T>::get();
		let new = seed_trust_validators
			.iter()
			.take(seed_trust_slots as usize)
			.cloned()
			.collect::<Vec<_>>();
		let old = SeedTrustValidators::<T>::get();
		// ToDo: Maybe this should be sorted
		if old == new {
			Self::deposit_event(Event::<T>::ValidatorsNotChanged);
			return old
		}
		SeedTrustValidators::<T>::put(&new);
		Self::deposit_event(Event::<T>::SeedTrustValidatorsElected { validators: new.clone() });
		new
	}

	fn do_elect_pot_validators(era_index: EraIndex, num_pot: u32) -> Vec<T::AccountId> {
		// PoT election phase
		log!(trace, "Elect pot validators at era {:?}", era_index);
		let mut voting_status = PotValidatorPool::<T>::get();
		voting_status.sort_by_vote_points();
		let new = voting_status.top_validators(num_pot).clone();
		let old = PotValidators::<T>::get();
		if new.is_empty() {
			Self::deposit_event(Event::<T>::EmptyPotValidatorPool);
			return new
		}
		// ToDo: Maybe this should be sorted
		if old == new {
			Self::deposit_event(Event::<T>::ValidatorsNotChanged);
			return old
		}
		PotValidators::<T>::put(&new);
		Self::deposit_event(Event::<T>::PotValidatorsElected { validators: new.clone() });
		new
	}

	/// Helper to set a new `ForceEra` mode.
	pub fn set_force_era(mode: Forcing) {
		log!(debug, "Setting force era mode {:?}.", mode);
		ForceEra::<T>::put(mode);
		Self::deposit_event(Event::<T>::ForceEra { mode });
	}

	pub fn try_set_number_of_validator(
		new_total_slots: u32,
		maybe_new_seed_trust_slots: Option<u32>,
	) -> sp_runtime::DispatchResult {
		let current_seed_trust_slots = SeedTrustSlots::<T>::get();
		// 1. Check if 'new_total_slots' is smaller than 'current_seed_trust_slots',
		//    'new_seed_trust_slots' should be provided
		if new_total_slots < current_seed_trust_slots {
			frame_support::ensure!(
				!maybe_new_seed_trust_slots.is_none(),
				Error::<T>::SeedTrustSlotsShouldBeProvided
			);
		}
		// 2. Set 'total_validator_slots'
		TotalValidatorSlots::<T>::put(new_total_slots);
		Self::deposit_event(Event::<T>::TotalValidatorSlotsChanged { new: new_total_slots });
		// 3. Do something if `new_seed_trust_slots` is provided
		if let Some(new_seed_trust_slots) = maybe_new_seed_trust_slots {
			frame_support::ensure!(
				new_total_slots >= new_seed_trust_slots,
				Error::<T>::SeedTrustExceedMaxValidators
			);
			SeedTrustSlots::<T>::put(new_seed_trust_slots);
			Self::deposit_event(Event::<T>::SeedTrustSlotsChanged { new: new_seed_trust_slots });
		}
		Ok(())
	}

	fn is_pot_enabled(num_pot: u32) -> bool {
		if num_pot > 0 && Self::pool_status() == Pool::All {
			return true
		}
		false
	}
}
