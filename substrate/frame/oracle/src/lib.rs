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

use frame_support::pallet_prelude::*;
use frame_system::{
	offchain::{SendTransactionTypes, SubmitTransaction},
	pallet_prelude::*,
};
use lite_json::JsonValue;
use sp_runtime::{infra::*, offchain::http, traits::Zero, traits::AtLeast32BitUnsigned};
use sp_std::{prelude::ToOwned, vec::Vec};

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type SystemTokenOracle: SystemTokenOracleInterface;

		/// Type of SystemToken
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		type SystemConfig: RuntimeConfigProvider<Self::Balance>;

		#[pallet::constant]
		type RequestPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
	}

	/// List of Fiat that should be requested via offchain call
	#[pallet::storage]
	pub type RequestedFiat<T: Config> = StorageValue<_, Vec<Fiat>>;

	/// Standard exchange rate time based on unix timestamp
	#[pallet::storage]
	pub type RequestStandardTime<T: Config> = StorageValue<_, StandardUnixTime, ValueQuery>;

	/// Exhange rate for each currency
	#[pallet::storage]
	pub type ExchangeRates<T: Config> =
		StorageMap<_, Twox64Concat, Fiat, ExchangeRate, OptionQuery>;

	/// System Tokens registered in the system for fetching exchange rates
	#[pallet::storage]
	pub type SystemTokens<T: Config> = StorageValue<_, Fiat>;

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			if let Call::submit_exchange_rates_unsigned { .. } = call {
				// TODO: Needs to add some validity check for the transaction
				// - Make it signed payload

				ValidTransaction::with_tag_prefix("OffchainWorker")
					.priority(T::UnsignedPriority::get())
					.longevity(5)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fiat requested from RC
		FiatRequested { fiat: Vec<Fiat> },
		/// Exchange rates submitted
		ExchangeRatesSubmitted { exchange_rates: Vec<(Fiat, ExchangeRate)> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The currency is not found.
		CurrencyNotFound,
		/// Error on parsing
		ParseError,
		/// System Config is missing
		SystemConfigMissing,
		/// Conversion Error
		ConversionError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			if block_number % T::RequestPeriod::get() == Zero::zero() {
				// We only request exchange rate if there is any fiat requested from RC
				if let Some(fiats) = RequestedFiat::<T>::get() {
					if let Err(_) = Self::fetch_exchange_rate(fiats.clone()) {
						log::warn!("❌❌ Failed to fetch exchange rate for => {:?}", fiats);
					}
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
			exchange_rates: Vec<(Fiat, ExchangeRate)>,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			T::SystemTokenOracle::submit_exchange_rates(exchange_rates.clone());
			Self::deposit_event(Event::<T>::ExchangeRatesSubmitted { exchange_rates });
			Ok(())
		}

		#[pallet::call_index(1)]
		pub fn request_fiat(
			origin: OriginFor<T>,
			fiat: Vec<Fiat>, 
		) -> DispatchResult {
			ensure_root(origin)?;
			for f in fiat.iter() {
				RequestedFiat::<T>::mutate(|maybe_fiats| {
					let mut fiats = maybe_fiats.take().unwrap_or_default();
					if !fiats.contains(&f) {
						fiats.push(f.clone());
					}
					*maybe_fiats = Some(fiats);
				});
			}
			Self::deposit_event(Event::<T>::FiatRequested { fiat });
			Ok(())
		}
		
		#[pallet::call_index(2)]
		pub fn add_oracle(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Ok(())
		}
	}
}

// utils
impl<T: Config> Pallet<T> {
	fn get_url(fiat: Fiat) -> Result<Vec<u8>, DispatchError> {
		let base_fiat = T::SystemConfig::system_config().map_err(|_| Error::<T>::SystemConfigMissing)?.base_system_token_detail.base_currency;
		let base_fiat_bytes: Vec<u8> = base_fiat.try_into().map_err(|_| Error::<T>::ConversionError)?;
		let fiat_to_bytes: Vec<u8> = fiat.try_into().map_err(|_| Error::<T>::ConversionError)?;
		let mut url: Vec<u8> = Vec::new();
		url.extend_from_slice(API_END_POINT.as_bytes());
		url.extend_from_slice(&base_fiat_bytes[..]);
		url.extend_from_slice("/".as_bytes());
		url.extend_from_slice(&fiat_to_bytes[..]);
		Ok(url)
	}
}

// ocw
impl<T: Config> Pallet<T> {

	/// Fetch the exchange rate from the oracle.
	fn fetch_exchange_rate(fiats: Vec<Fiat>) -> Result<(), http::Error> {
		let mut exchange_rates: Vec<(Fiat, ExchangeRate)> = Vec::new();
		for fiat in fiats {
			let res = Self::do_fetch(fiat)?;
			exchange_rates.push(res);
		}
		let call = Call::submit_exchange_rates_unsigned { exchange_rates };
		if let Err(_) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
			log::error!("❌❌ Failed to submit exchange rates.");
		} 
		Ok(())
	}

	fn do_fetch(fiat: Fiat) -> Result<(Fiat, ExchangeRate), http::Error> {
		let url_bytes = Self::get_url(fiat).map_err(|_| http::Error::Unknown)?;
		let url = sp_std::str::from_utf8(&url_bytes).map_err(|_| http::Error::Unknown)?;
		log::info!("😈😈 Requesting URL => {:?}", url);
		let deadline =
			sp_io::offchain::timestamp().add(sp_core::offchain::Duration::from_millis(2000));
		let request = sp_runtime::offchain::http::Request::get(url);
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
		Ok(
			Self::parse_json(body_str).map_err(|_| {
				log::warn!("Failed to parse json");
				http::Error::Unknown
			})?
		)
	}

	fn parse_json(exchange_rate_str: &str) -> Result<(Fiat, ExchangeRate), DispatchError> {
		let val = lite_json::parse_json(exchange_rate_str);
		if let Ok(val) = val {
			match val {
				JsonValue::Object(obj) => {
					// let standard_time = Self::standard_time(&obj)?;

					return Ok(Self::exchange_rate_for(&obj)?)
				},
				_ => return Err(Error::<T>::ParseError.into()),
			};
		} else {
			Err(Error::<T>::ParseError.into())
		}
	}

	fn _standard_time(obj: &Vec<(Vec<char>, JsonValue)>) -> Result<StandardUnixTime, DispatchError> {
		if let Some((_, v)) =
			obj.iter().find(|(k, _)| k.iter().copied().eq("time_last_update_unix".chars()))
		{
			match v {
				JsonValue::Number(n) => {
					log::info!("Standard Time => {:?}", n.integer);
					return Ok(n.integer)
				},
				_ => return Err(Error::<T>::ParseError.into()),
			}
		} else {
			return Err(Error::<T>::ParseError.into())
		}
	}

	fn exchange_rate_for(
		obj: &Vec<(Vec<char>, JsonValue)>,
	) -> Result<(Fiat, ExchangeRate), DispatchError> {
		let mut exchange_rate = None;
		let mut target_code = None;
	
		for (k, v) in obj.iter() {
			if k.iter().copied().eq("conversion_rate".chars()) {
				if let JsonValue::Number(n) = v {
					exchange_rate = Some(n.integer);
				} else {
					return Err(Error::<T>::ParseError.into());
				}
			} else if k.iter().copied().eq("target_code".chars()) {
				if let JsonValue::String(s) = v {
					let byte_vec = s
						.iter()
						.flat_map(|&c| c.encode_utf8(&mut [0; 4]).as_bytes().to_owned())
						.collect::<Vec<u8>>();
					let fiat: Fiat = byte_vec.try_into().map_err(|_| Error::<T>::CurrencyNotFound)?;
					target_code = Some(fiat);
				} else {
					return Err(Error::<T>::ParseError.into());
				}
			}
		}
	
		let exchange_rate = exchange_rate.ok_or(Error::<T>::ParseError)?;
		let target_code = target_code.ok_or(Error::<T>::ParseError)?;
		Ok((target_code, exchange_rate))
	}

	fn _exchange_rates_all(
		obj: &Vec<(Vec<char>, JsonValue)>,
	) -> Result<Vec<(Fiat, ExchangeRate)>, DispatchError> {
		let exchange_obj = if let Some((_, v)) =
			obj.into_iter().find(|(k, _)| k.iter().copied().eq("conversion_rates".chars()))
		{
			match v {
				JsonValue::Object(obj) => obj,
				_ => return Err(Error::<T>::ParseError.into()),
			}
		} else {
			return Err(Error::<T>::ParseError.into())
		};
		let mut exchange_rates: Vec<(Fiat, ExchangeRate)> = Vec::new();
		for (k, v) in exchange_obj.into_iter() {
			let byte_vec = k
				.iter()
				.flat_map(|&c| c.encode_utf8(&mut [0; 4]).as_bytes().to_owned())
				.collect::<Vec<u8>>();
			match v {
				JsonValue::Number(n) => {
					let fiat: Fiat =
						byte_vec.try_into().map_err(|_| Error::<T>::CurrencyNotFound)?;
					exchange_rates.push((fiat, n.integer));
				},
				_ => return Err(Error::<T>::ParseError.into()),
			};
		}
		Ok(exchange_rates)
	}
}
