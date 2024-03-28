// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod migrations;

pub mod impls;
pub use impls::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{
		tokens::{fungibles::{Inspect, InspectSystemToken, Mutate}, Balance},
		EstimateNextNewSession, Get,
	},
	Parameter,
};
pub use pallet::*;
use scale_info::TypeInfo;
use softfloat::BlockTimeWeight;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{
	traits::Member,
	types::{infra_core::TaaV, vote::PotVote},
	RuntimeDebug,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod mock;

use sp_std::prelude::*;

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

pub type SystemTokenAssetIdOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type SystemTokenBalanceOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub(crate) const LOG_TARGET: &str = "runtime::voting-manager";
// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 🗳️ ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

/// Compose of validator pool
#[derive(
	Copy,
	Clone,
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
pub enum Pool {
	// Seed Trust & PoT validators will be elected
	All,
	// Only Seed Trust validators will be elected
	SeedTrust,
}

impl Default for Pool {
	fn default() -> Self {
		Pool::SeedTrust
	}
}

#[derive(
	Copy,
	Clone,
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
pub enum Forcing {
	/// Not forcing anything - just let whatever happen.
	NotForcing,
	/// Force a new era, then reset to `NotForcing` as soon as it is done.
	/// Note that this will force to trigger an election until a new era is triggered, if the
	/// election failed, the next session end will trigger a new election again, until success.
	ForceNew,
	/// Avoid a new era indefinitely.
	ForceNone,
	/// Force a new era at the end of all sessions indefinitely.
	ForceAlways,
}

impl Default for Forcing {
	fn default() -> Self {
		Forcing::NotForcing
	}
}

#[derive(
	Copy,
	Clone,
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct Reward<AssetId, Amount> {
	asset: AssetId,
	amount: Amount,
}

impl<AssetId, Amount: Balance> Reward<AssetId, Amount> {
	pub fn new(asset: AssetId) -> Self {
		Self { asset, amount: Default::default() }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingStatus<T: Config> {
	pub status: Vec<(T::AccountId, T::HigherPrecisionScore)>,
}

impl<T: Config> Default for VotingStatus<T> {
	fn default() -> Self {
		Self { status: Default::default() }
	}
}

impl<T: Config> VotingStatus<T> {
	/// Add vote point for given vote account id and vote points.
	pub fn add_vote(&mut self, who: &T::AccountId, vote_weight: T::HigherPrecisionScore) {
		for (candidate, amount) in self.status.iter_mut() {
			if candidate == who {
				*amount += vote_weight;
				return
			}
		}
		self.status.push((who.clone(), vote_weight));
	}

	pub fn counts(&self) -> usize {
		self.status.len()
	}

	/// Sort vote status for decreasing order
	pub fn sort_by_vote_points(&mut self) {
		self.status.sort_by(|x, y| {
			let vote1: T::Score = x.1.clone().into();
			let vote2: T::Score = y.1.clone().into();
			vote2.cmp(&vote1)
		});
	}

	/// Get top validators for given vote status.
	/// We elect validators based on PoT which has exceeded the minimum vote points.
	///
	/// Note:
	/// This function should be called after `sort_by_vote_points` is called.
	pub fn top_validators(&mut self, num: u32) -> Vec<T::AccountId> {
		self.status
			.iter()
			.take(num as usize)
			.filter(|vote_status| {
				let vote: T::Score = vote_status.1.clone().into();
				vote >= MinVotePointsThreshold::<T>::get()
			})
			.map(|vote_status| vote_status.0.clone())
			.collect()
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Number of sessions per era.
		#[pallet::constant]
		type SessionsPerEra: Get<SessionIndex>;
		
		// F64::from_i128(5_256_000)(e.g 10 blocks/min * 60 min/hours* 24 hours/day * 365 days/year)
		/// The number of blocks per year
		#[pallet::constant]
		type BlocksPerYear: Get<BlockNumberFor<Self>>;

		/// Local fungibles trait
		type Fungibles: InspectSystemToken<Self::AccountId>;

		/// Type that handles aggregated reward
		type RewardHandler: RewardInterface<AccountId=Self::AccountId, AssetKind=SystemTokenAssetIdOf<Self>, Balance=Self::Score>;

		/// Associated type for vote weight
		type Score: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Copy
			+ Default
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ From<BlockNumberFor<Self>>
			+ Into<Self::HigherPrecisionScore>;

		/// A type used for calculations of `Score` with higher precision to store on chain
		/// TODO:
		type HigherPrecisionScore: BlockTimeWeight<Self::Score, BlockNumberFor<Self>>
			+ Parameter
			+ Member
			+ Into<Self::Score>;

		/// Something that can estimate the next session change, accurately or as a best effort
		/// guess.
		type NextNewSession: EstimateNextNewSession<BlockNumberFor<Self>>;

		/// Interface for interacting with a session pallet.
		type SessionInterface: SessionInterface<Self::AccountId>;

		/// Interface for interacting with validator collective pallet
		type CollectiveInterface: CollectiveInterface<Self::AccountId>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub seed_trust_validators: Vec<T::AccountId>,
		pub total_validator_slots: u32,
		pub seed_trust_slots: u32,
		pub force_era: Forcing,
		pub pool_status: Pool,
		pub is_pot_enable_at_genesis: bool,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				is_pot_enable_at_genesis: false,
				total_validator_slots: Default::default(),
				seed_trust_validators: Default::default(),
				seed_trust_slots: Default::default(),
				force_era: Default::default(),
				pool_status: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(self.seed_trust_slots <= self.total_validator_slots);
			SeedTrustValidatorPool::<T>::put(self.seed_trust_validators.clone());
			TotalValidatorSlots::<T>::put(self.total_validator_slots.clone());
			SeedTrustSlots::<T>::put(self.seed_trust_slots.clone());
			ForceEra::<T>::put(self.force_era);
			PoolStatus::<T>::put(self.pool_status);
			if self.is_pot_enable_at_genesis {
				// do_something
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Points has been added for candidate validator
		Voted { who: T::AccountId, amount: T::Score },
		/// Total number of validators has been changed
		TotalValidatorSlotsChanged { new: u32 },
		/// Number of seed trust validators has been changed
		SeedTrustSlotsChanged { new: u32 },
		/// Seed trust validator has been added to the pool
		SeedTrustAdded { who: T::AccountId },
		/// Validator have been elected
		ValidatorsElected { validators: Vec<T::AccountId>, pot_enabled: bool },
		/// Seed Trust validators have been elected
		SeedTrustValidatorsElected { validators: Vec<T::AccountId> },
		/// Validators have been elected by PoT
		PotValidatorsElected { validators: Vec<T::AccountId> },
		/// Min vote weight has been set
		MinVotePointsChanged { old: T::Score, new: T::Score },
		/// If new validator set is same as old validator. This could be caused by seed trust/pot
		/// election.
		ValidatorsNotChanged,
		/// When there is no candidate validator in PotValidatorPool
		EmptyPotValidatorPool,
		/// A new force era mode was set.
		ForceEra { mode: Forcing },
		/// New era has triggered
		NewEraTriggered { era_index: EraIndex },
		/// New pool status has been set
		PoolStatusSet { status: Pool },
		/// Rewarded for validator
		Rewarded { at_era: EraIndex, asset: SystemTokenAssetIdOf<T>, amount: T::Score },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Total validators num should be greater or equal to number of current validators
		LessThanCurrentValidatorsNum,
		/// Number of seed trust validators should be less or equal to total number of validators
		SeedTrustExceedMaxValidators,
		/// Some parameters for transaction are bad
		BadTransactionParams,
		/// New number of Seed Trust slots should be provided
		SeedTrustSlotsShouldBeProvided,
		/// Error occured while decoding types mostly `PotVote`
		ErrorDecode,
	}

	/// The current era index.
	///
	/// This is the latest planned era, depending on how the Session pallet queues the validator
	/// set, it might be active or not.
	#[pallet::storage]
	pub type CurrentEra<T> = StorageValue<_, EraIndex, OptionQuery>;

	// Pot pool that tracks all the candidate validators who have been voted
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PotValidatorPool<T: Config> = StorageValue<_, VotingStatus<T>, ValueQuery>;

	// Candidate Seed Trust validators set
	#[pallet::storage]
	#[pallet::unbounded]
	pub type SeedTrustValidatorPool<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Current Seed Trust validators
	#[pallet::storage]
	#[pallet::unbounded]
	pub type SeedTrustValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Cuurent validators which have been elected by PoT
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PotValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Number of seed trust validators that can be elected
	#[pallet::storage]
	pub type NumberOfSeedTrustValidators<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Number of seed trust validators that can be elected
	#[pallet::storage]
	pub type SeedTrustSlots<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Total Number of validators that can be elected,
	/// which is composed of seed trust validators and pot validators
	#[pallet::storage]
	pub type TotalValidatorSlots<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type MinVotePointsThreshold<T: Config> = StorageValue<_, T::Score, ValueQuery>;

	/// Start Session index for era
	#[pallet::storage]
	pub type StartSessionIndexPerEra<T: Config> =
		StorageMap<_, Twox64Concat, EraIndex, SessionIndex, OptionQuery>;

	/// Mode of era forcing
	#[pallet::storage]
	#[pallet::getter(fn force_era)]
	pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery>;

	/// Mode of validator pool
	#[pallet::storage]
	#[pallet::getter(fn pool_status)]
	pub type PoolStatus<T> = StorageValue<_, Pool, ValueQuery>;

	/// Reward for each validator
	#[pallet::storage]
	#[pallet::unbounded]
	pub type RewardInfo<T: Config> = StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, T::AccountId, Vec<Reward<SystemTokenAssetIdOf<T>, T::Score>>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn set_number_of_validators(
			origin: OriginFor<T>,
			new_total_slots: u32,
			new_seed_trust_slots: Option<u32>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::try_set_number_of_validator(new_total_slots, new_seed_trust_slots)?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn add_seed_trust_validator(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			let mut seed_trust_validators = SeedTrustValidatorPool::<T>::get();
			seed_trust_validators.push(who.clone());
			SeedTrustValidatorPool::<T>::put(seed_trust_validators);
			Self::deposit_event(Event::<T>::SeedTrustAdded { who });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn set_min_vote_weight_threshold(
			origin: OriginFor<T>,
			new: T::Score,
		) -> DispatchResult {
			ensure_root(origin)?;
			let old = MinVotePointsThreshold::<T>::get();
			MinVotePointsThreshold::<T>::put(new);
			Self::deposit_event(Event::<T>::MinVotePointsChanged { old, new });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn set_pool_status(origin: OriginFor<T>, status: Pool) -> DispatchResult {
			ensure_root(origin)?;
			PoolStatus::<T>::put(status);
			Self::deposit_event(Event::<T>::PoolStatusSet { status });

			Ok(())
		}
	}
}
