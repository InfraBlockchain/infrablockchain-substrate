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

use lite_json::JsonValue;
use frame_support::{error::BadOrigin, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use pallet_session::ShouldEndSession;
use sp_runtime::{
	types::{SystemTokenWeight, AssetId as SystemTokenAssetId},
	offchain::http
};
use sp_std::{vec::Vec, prelude::ToOwned};

pub use pallet::*;

const LOG_TARGET: &str = "runtime::system-token-helper";
const REQUEST_API: &str = "https://v6.exchangerate-api.com/v6/b17c41b872d0b8a2efd77e08/latest/USD";

/// Ensure that the origin `o` represents a relay chain.
/// Returns `Ok` that effected the extrinsic or an `Err` otherwise.
pub fn ensure_system_token_origin<OuterOrigin>(o: OuterOrigin) -> Result<(), BadOrigin>
where
	OuterOrigin: Into<Result<Origin, OuterOrigin>>,
{
	match o.into() {
		Ok(Origin::SystemTokenBody) => Ok(()),
		_ => Err(BadOrigin),
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {

		type ShouldEndSession: ShouldEndSession<BlockNumberFor<Self>>;
		/// The base weight of system token for this Runtime.
		/// Usually, this refers to USD.
		#[pallet::constant]
		type BaseWeight: Get<SystemTokenWeight>;
	}

	/// Weight will be defined whenever registered as System Token
	#[pallet::storage]
	pub type Weights<T: Config> = StorageMap<_, Blake2_128Concat, SystemTokenAssetId, SystemTokenWeight, OptionQuery>;

	/// Origin for the parachains.
	#[pallet::origin]
	#[derive(
		PartialEq,
		Eq,
		Clone,
		Encode,
		Decode,
		sp_core::RuntimeDebug,
		scale_info::TypeInfo,
		MaxEncodedLen,
	)]
	pub enum Origin {
		/// It comes from a relay chain
		SystemTokenBody,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			if T::ShouldEndSession::should_end_session(block_number) {
				if let Ok(_) = Self::fetch_exchange_rate() {

				} else {
					log::warn!(target: LOG_TARGET, "Failed to fetch exchange rate")
				}
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Fetch the exchange rate from the oracle.
	fn fetch_exchange_rate() -> Result<(), http::Error> {
		let deadline = sp_io::offchain::timestamp().add(sp_core::offchain::Duration::from_millis(2_000));
		let request =
			sp_runtime::offchain::http::Request::get(REQUEST_API);
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
		}
		let body = response.body().collect::<Vec<u8>>();
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;
		Self::parse_json(body_str);
		Ok(())
	}

	fn parse_json(exchange_rate_str: &str) {
		let val = lite_json::parse_json(exchange_rate_str);
		if let Ok(val) = val {
			let exchange_obj = match val {
				JsonValue::Object(obj) => {
					if let Some((_, v)) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("conversion_rates".chars())) {
						match v {
							JsonValue::Object(obj) => obj,
							_ => return
						}
					} else {
						return
					}
				},
				_ => return,
			};
			for (k, v) in exchange_obj.into_iter() {
				let byte_vec = k.iter().flat_map(|&c| c.encode_utf8(&mut [0; 4]).as_bytes().to_owned()).collect::<Vec<u8>>();
				let rate = match v {
					JsonValue::Number(n) => n,
					_ => return,
				};
				log::info!("{:?} : {:?}", byte_vec, rate.integer);
			}
		} else {
			return;
		}
	}
}

pub trait SystemTokenHelper {
	/// The base weight of system token for this Runtime.
	/// Usually, this refers to USD.
	fn base_weight() -> SystemTokenWeight;

	fn weight_for(id: &SystemTokenAssetId) -> SystemTokenWeight;
}

impl<T: Config> SystemTokenHelper for pallet::Pallet<T> {
	fn base_weight() -> SystemTokenWeight {
		T::BaseWeight::get()
	}

	fn weight_for(id: &SystemTokenAssetId) -> SystemTokenWeight {
		Weights::<T>::get(id).map_or(Self::base_weight(), |w| w)
	}
}
