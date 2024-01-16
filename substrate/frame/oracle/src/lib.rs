// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the License);
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an AS IS BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

mod types;
pub use types::*;

use lite_json::JsonValue;
use frame_support::{error::BadOrigin, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_system::offchain::{
	SubmitTransaction, SendTransactionTypes
};
use sp_runtime::{types::SystemTokenWeight, offchain::http, traits::Zero};
use sp_std::{vec::Vec, prelude::ToOwned};

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {

		type SystemTokenOracle: SystemTokenOracleInterface;

		#[pallet::constant]
		type RequestPeriod: Get<BlockNumberFor<Self>>;
		/// The base weight of system token for this Runtime.
		/// Usually, this refers to USD.
		#[pallet::constant]
		type BaseWeight: Get<SystemTokenWeight>;

		#[pallet::constant]
		type IsOffChain: Get<bool>;

		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
	}

	/// Standard exchange rate time based on unix timestamp
	#[pallet::storage]
	pub type RequestStandardTime<T: Config> = StorageValue<_, StandardUnixTime, ValueQuery>;

	/// Exhange rate for each currency
	#[pallet::storage]
	pub type ExchangeRates<T: Config> = StorageMap<_, Twox64Concat, Fiat, ExchangeRate, OptionQuery>;

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			if let Call::submit_exchange_rates_unsigned { standard_time, .. } = call {
				
				// TODO: Needs to add some validity check for the transaction
				// - Make it signed payload

				ValidTransaction::with_tag_prefix("OffchainWorker")
					.priority(T::UnsignedPriority::get())
					.and_provides(standard_time)
					.longevity(5)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The currency is not found.
		CurrencyNotFound,
		/// Error on parsing
		ParseError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> 
	{
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			if block_number % T::RequestPeriod::get() == Zero::zero() && T::IsOffChain::get() {
				if let Ok(_) = Self::fetch_exchange_rate() {

				} else {
					log::warn!(target: LOG_TARGET, "Failed to fetch exchange rate")
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit unsigned extrinsic to set the exchange rates.
		/// It is an open door to Runtime. So, we should figure out how to make it secure.
		#[pallet::call_index(0)]
		pub fn submit_exchange_rates_unsigned(
			origin: OriginFor<T>,
			standard_time: StandardUnixTime,
			exchange_rates: Vec<(Fiat, ExchangeRate)>,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			
			T::SystemTokenOracle::exchange_rates_at(standard_time, exchange_rates);

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Fetch the exchange rate from the oracle.
	fn fetch_exchange_rate() -> Result<(), http::Error> {
		let deadline = sp_io::offchain::timestamp().add(sp_core::offchain::Duration::from_millis(2000));
		let request =
			sp_runtime::offchain::http::Request::get(API_END_POINT);
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
		Self::parse_json(body_str).map_err(|_| {
			log::warn!("Failed to parse json");
			http::Error::Unknown
		})?;
		Ok(())
	}

	fn parse_json(exchange_rate_str: &str) -> Result<(), DispatchError>{
		let val = lite_json::parse_json(exchange_rate_str);
		if let Ok(val) = val {
			match val {
				JsonValue::Object(obj) => {
					let standard_time = Self::standard_time(&obj)?;
					let exchange_rates = Self::exchange_rates(&obj)?;

					let call = Call::submit_exchange_rates_unsigned { standard_time, exchange_rates };
					SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
						.map_err(|()| "Unable to submit unsigned transaction.")?;

					return Ok(())
				},
				_ => return Err(Error::<T>::ParseError.into()),
			};
		} else {
			Err(Error::<T>::ParseError.into())
		}
	}

	fn standard_time(obj: &Vec<(Vec<char>, JsonValue)>) -> Result<StandardUnixTime, DispatchError>{
		if let Some((_, v)) = obj.iter().find(|(k, _)| k.iter().copied().eq("time_last_update_unix".chars())) {
			match v {
				JsonValue::Number(n) => {
					log::info!("Standard Time => {:?}", n.integer);
					return Ok(n.integer)
				},
				_ => return Err(Error::<T>::ParseError.into())
			}
		} else {
			return Err(Error::<T>::ParseError.into())
		}
	}

	fn exchange_rates(obj: &Vec<(Vec<char>, JsonValue)>) -> Result<Vec<(Fiat, ExchangeRate)>, DispatchError> {
		let exchange_obj = if let Some((_, v)) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("conversion_rates".chars())) {
			match v {
				JsonValue::Object(obj) => obj,
				_ => return Err(Error::<T>::ParseError.into())
			}
		} else {
			return Err(Error::<T>::ParseError.into())
		};
		let mut exchange_rates: Vec<(Fiat, ExchangeRate)> = Vec::new();
		for (k, v) in exchange_obj.into_iter() {
			let byte_vec = k.iter().flat_map(|&c| c.encode_utf8(&mut [0; 4]).as_bytes().to_owned()).collect::<Vec<u8>>();
			match v {
				JsonValue::Number(n) => {
					let fiat: Fiat = byte_vec.try_into().map_err(|_| Error::<T>::CurrencyNotFound)?;
					log::info!("Currency => {:?}, Rates => {:?}", fiat, n.integer);
					exchange_rates.push((fiat, n.integer));
				},
				_ => return Err(Error::<T>::ParseError.into()),
			};
		}
		Ok(exchange_rates)
	}
}

