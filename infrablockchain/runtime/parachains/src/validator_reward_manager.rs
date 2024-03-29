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

//! # Pot Reward Pallet
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Pot Reward Pallet is a pallet that rewards validators
//! who are selected due to pot consensus.

// impl me!

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{IsType, ValidatorSet, tokens::fungibles::{InspectSystemToken, Inspect}},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_validator_election::{RewardInterface, SessionIndex};
use scale_info::TypeInfo;
use softfloat::F64;
use sp_runtime::{
	traits::{Convert, StaticLookup},
	types::{token::*, vote::*},
};
use primitives::{Id as ParaId};
use sp_std::prelude::*;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub type SystemTokenAssetIdOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type SystemTokenBalanceOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// A type for representing the validator id in a session.
pub type ValidatorId<T> = <<T as Config>::ValidatorSet as ValidatorSet<
	<T as frame_system::Config>::AccountId,
>>::ValidatorId;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ValidatorReward<AssetId, Balance> {
	pub system_token_id: AssetId,
	pub amount: Balance,
}

impl<AssetId, Balance> ValidatorReward<AssetId, Balance> {
	pub fn new(system_token_id: AssetId, amount: Balance) -> Self {
		Self { system_token_id, amount }
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::{configuration, dmp, paras, system_token_helper};

	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ configuration::Config
		+ paras::Config
		+ dmp::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type for retrieving the validators supposed to be online in a session.
		type ValidatorSet: ValidatorSet<Self::AccountId>;
		/// Local fungibles module
		type Fungibles: InspectSystemToken<Self::AccountId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn validator_rewards)]
	#[pallet::unbounded]
	pub type ValidatorRewards<T: Config> =
		StorageMap<_, Twox64Concat, ValidatorId<T>, Vec<ValidatorReward<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>>>;

	#[pallet::storage]
	#[pallet::getter(fn session_rewards)]
	#[pallet::unbounded]
	pub type TotalSessionRewards<T: Config> =
		StorageMap<_, Twox64Concat, SessionIndex, Vec<ValidatorReward<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>>>;

	#[pallet::storage]
	#[pallet::getter(fn rewards_by_parachain)]
	#[pallet::unbounded]
	pub type RewardsByParaId<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		SessionIndex,
		Twox64Concat,
		ParaId,
		Vec<ValidatorReward<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The validator has been rewarded.
		ValidatorRewarded { stash: ValidatorId<T>, system_token_id: SystemTokenAssetIdOf<T>, amount: SystemTokenBalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not a controller account.
		NotController,
		/// Rewards already been claimed for this validator.
		AlreadyClaimed,
		EmptyAggregatedRewards,
		NothingToClaim,
		NeedOriginSignature,
		NoAssociatedValidatorId,
		ExceedsMaxMessageSize,
		Unknown,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	{
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn claim(
			origin: OriginFor<T>,
			validator: AccountIdLookupOf<T>,
			system_token_id: SystemTokenId,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let validator = T::Lookup::lookup(validator.clone())?;

			ensure!(origin == validator, Error::<T>::NeedOriginSignature);

			let who = <T::ValidatorSet as ValidatorSet<T::AccountId>>::ValidatorIdOf::convert(
				validator.clone(),
			)
			.ok_or(Error::<T>::NoAssociatedValidatorId)?;
			ensure!(ValidatorRewards::<T>::contains_key(who.clone()), Error::<T>::NothingToClaim);
			let mut rewards: Vec<ValidatorReward> =
				ValidatorRewards::<T>::get(who.clone()).unwrap_or_default();
			ensure!(rewards.len() != 0, Error::<T>::NothingToClaim);

			let sovereign = system_token_helper::sovereign_account::<T>();
			if let Some(reward) =
				rewards.iter_mut().find(|ar| ar.system_token_id == system_token_id)
			{
				let SystemTokenId { para_id, pallet_id, asset_id } = system_token_id;
				let amount: u128 = reward.amount.to_i128() as u128;
				let encoded_call: Vec<u8> = pallet_assets::Call::<T>::force_transfer2 {
					id: asset_id.into(),
					source: T::Lookup::unlookup(sovereign.clone()),
					dest: T::Lookup::unlookup(validator.clone()),
					amount: <T as pallet_assets::Config>::Balance::from(amount),
				}
				.encode();
				// TODO: impl me! InfraCore::reward_validator(para_id, pallet_id)
				// system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;
				Self::deposit_event(Event::ValidatorRewarded {
					stash: who.clone().into(),
					system_token_id,
					amount,
				});
				reward.amount = F64::from_i128(0);
			}
			ValidatorRewards::<T>::insert(who.clone(), rewards.clone());

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn aggregate_reward(
		session_index: SessionIndex,
		para_id: SystemTokenParaId,
		system_token_id: SystemTokenId,
		amount: VoteWeight,
	) {
		if let Some(mut rewards) = RewardsByParaId::<T>::get(session_index, para_id.clone()) {
			for reward in rewards.iter_mut() {
				if reward.system_token_id == system_token_id {
					reward.amount = reward.amount.add(amount);
				}
			}
			RewardsByParaId::<T>::insert(session_index, para_id.clone(), rewards.clone());
		} else {
			let rewards = vec![ValidatorReward::new(system_token_id, amount)];
			RewardsByParaId::<T>::insert(session_index, para_id.clone(), rewards);
		}

		if let Some(mut rewards) = TotalSessionRewards::<T>::get(session_index) {
			for reward in rewards.iter_mut() {
				if reward.system_token_id == system_token_id {
					reward.amount += amount;
				}
			}
			TotalSessionRewards::<T>::insert(session_index, rewards.clone());
		} else {
			let rewards = vec![ValidatorReward::new(system_token_id, amount)];
			TotalSessionRewards::<T>::insert(session_index, rewards);
		}
	}

	fn distribute_reward(session_index: SessionIndex) {
		let current_validators = T::ValidatorSet::validators();
		let current_validators_len = F64::from_i128(current_validators.len() as i128);
		let aggregated_rewards = TotalSessionRewards::<T>::get(session_index).unwrap_or_default();

		if aggregated_rewards.is_empty() {
			return
		}

		for validator in current_validators.iter() {
			if ValidatorRewards::<T>::contains_key(validator) {
				let mut rewards = ValidatorRewards::<T>::get(validator.clone()).unwrap_or_default();

				for aggregated_reward in aggregated_rewards.iter() {
					if let Some(reward) = rewards
						.iter_mut()
						.find(|ar| ar.system_token_id == aggregated_reward.system_token_id)
					{
						reward.amount += aggregated_reward.amount / current_validators_len
					} else {
						let new_reward = ValidatorReward::new(
							aggregated_reward.clone().system_token_id,
							aggregated_reward.clone().amount / current_validators_len,
						);
						rewards.push(new_reward);
					}
				}
				ValidatorRewards::<T>::insert(validator, rewards.clone());
			} else {
				let mut rewards: Vec<ValidatorReward> = vec![];
				for aggregated_reward in aggregated_rewards.iter() {
					let reward = ValidatorReward::new(
						aggregated_reward.clone().system_token_id,
						aggregated_reward.amount.div(current_validators_len),
					);
					rewards.push(reward);
				}
				ValidatorRewards::<T>::insert(validator, rewards.clone());
			}
		}
	}
}

// impl<T: Config> RewardInterface for Pallet<T> {
// 	fn aggregate_reward(
// 		session_index: SessionIndex,
// 		para_id: SystemTokenParaId,
// 		system_token_id: SystemTokenId,
// 		amount: VoteWeight,
// 	) {
// 		Self::aggregate_reward(session_index, para_id, system_token_id, amount);
// 	}

// 	fn distribute_reward(session_index: SessionIndex) {
// 		Self::distribute_reward(session_index);
// 	}
// }
