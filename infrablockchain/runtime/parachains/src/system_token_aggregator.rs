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

pub use crate::system_token_helper;
use frame_support::{
	pallet_prelude::*,
	traits::fungibles::{roles::Inspect, EnumerateSystemToken},
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use sp_runtime::{self, traits::Zero, types::token::*};
use sp_std::prelude::*;
use xcm::{latest::prelude::*, opaque::lts::MultiLocation};
use xcm_primitives::AssetMultiLocationGetter;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type Period: Get<BlockNumberFor<Self>>;
		type Fungibles: Inspect<Self::AccountId> + EnumerateSystemToken<Self::AccountId>;
		type AssetMultiLocationGetter: AssetMultiLocationGetter<SystemTokenAssetId>;
		type SendXcm: SendXcm;
		type IsRelay: Get<bool>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Aggrate System Token Successfully
		SystemTokenAggregated { asset_multi_loc: MultiLocation, amount: u128 },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if n % T::Period::get() == Zero::zero() {
				let is_relay = T::IsRelay::get();
				if let Err(e) = Self::do_aggregate_system_token(is_relay) {
					log::info!(
						target: "runtime::system_token_aggregator",
						"Aggregating system token failed. {:?}", e
					);
				}
				T::DbWeight::get().reads(3)
			} else {
				T::DbWeight::get().reads(0)
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub(crate) fn do_aggregate_system_token(is_relay: bool) -> Result<(), DispatchError> {
		let fee_account = system_token_helper::sovereign_account::<T>();
		for (asset, balance) in
			T::Fungibles::system_token_account_balances(fee_account.clone().into()).into_iter()
		{
			if let Some(asset_multi_loc) =
				T::AssetMultiLocationGetter::get_asset_multi_location(asset)
			// TODO: Remove
			{
				system_token_helper::do_teleport_asset::<T::AccountId, T::SendXcm>(
					&fee_account,
					&balance,
					&asset_multi_loc,
					is_relay,
				)?;

				Self::deposit_event(Event::<T>::SystemTokenAggregated {
					asset_multi_loc,
					amount: balance,
				});
			}
		}
		Ok(())
	}
}
