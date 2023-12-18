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

//! # Tokem manager Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! Token manager handles all infomration related with system tokens on the relay chain level.
//!
//! ### Functions
//!
//! * `set_name` - Set the associated name of an account; a small deposit is reserved if not already
//!   taken.
//! *

pub use crate::system_token_helper;
use frame_support::{pallet_prelude::*, traits::OriginTrait};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use sp_runtime::{
	self,
	traits::Zero,
	types::{AssetId, SystemTokenLocalAssetProvider},
};
use sp_std::prelude::*;
use xcm::opaque::lts::MultiLocation;
use xcm_primitives::AssetMultiLocationGetter;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_assets::Config
		+ pallet_xcm::Config
		+ pallet_asset_link::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type Period: Get<BlockNumberFor<Self>>;
		type AssetMultiLocationGetter: AssetMultiLocationGetter<AssetId>;
		type IsRelay: Get<bool>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Aggrate System Token Successfully
		SystemTokenAggregated { multilocation: MultiLocation, amount: u128 },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		u32: From<BlockNumberFor<T>>,
		<<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId:
			From<AccountIdOf<T>>,
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
		u128: From<<T as pallet_assets::Config>::Balance>,
	{
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if n % T::Period::get() == Zero::zero() {
				let is_relay = T::IsRelay::get();
				Self::do_aggregate_system_token(is_relay);
				T::DbWeight::get().reads(3)
			} else {
				T::DbWeight::get().reads(0)
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub(crate) fn do_aggregate_system_token(is_relay: bool)
	where
		u32: From<BlockNumberFor<T>>,
		<<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId:
			From<AccountIdOf<T>>,
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
		u128: From<<T as pallet_assets::Config>::Balance>,
	{
		let fee_account = system_token_helper::sovereign_account::<T>();
		let system_token_asset_list =
			pallet_assets::Pallet::<T>::system_token_list().map_or(Default::default(), |l| l);
		let balances = pallet_assets::Pallet::<T>::account_balances(fee_account.clone());
		for (asset_id, amount) in balances.iter() {
			if !system_token_helper::inspect_account_and_check_is_owner::<T>(&asset_id) ||
				!system_token_asset_list.contains(&asset_id.clone().into())
			{
				continue
			}

			if let Some(asset_multilocation) =
				T::AssetMultiLocationGetter::get_asset_multi_location(asset_id.clone().into())
			{
				system_token_helper::do_teleport_asset::<T>(
					fee_account.clone(),
					amount,
					asset_multilocation.clone(),
					is_relay,
				);

				Self::deposit_event(Event::<T>::SystemTokenAggregated {
					multilocation: asset_multilocation.clone(),
					amount: amount.clone().into(),
				});
			}
		}
	}
}
