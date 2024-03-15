// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{configuration, ensure_parachain, paras, Origin as ParachainOrigin, ParaId};
use frame_support::storage::KeyPrefixIterator;
pub use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungibles::{Inspect, InspectSystemToken},
			Balance, SystemTokenId,
		},
		UnixTime,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use softfloat::F64;
use sp_runtime::{types::{infra_core::*, token::*}, traits::Zero};
use sp_std::prelude::*;
use types::*;
pub use traits::SystemTokenInterface;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + configuration::Config + paras::Config {
		/// Origin for this module
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<ParachainOrigin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Local fungibles module
		type Fungibles: InspectSystemToken<Self::AccountId>;
		/// Id of System Token
		type SystemTokenId: SystemTokenId;
		/// Type for handling System Token related calls 
		type SystemTokenHandler: SystemTokenInterface;
		/// The string limit for name and symbol of system token.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// Max number of system tokens that can be used on parachain.
		#[pallet::constant]
		type MaxSystemTokens: Get<u32>;
		/// Max number of `paraId` that are using `original` system token
		#[pallet::constant]
		type MaxOriginalUsedParaIds: Get<u32>;
		/// The ParaId of the asset hub system parachain.
		#[pallet::constant]
		type AssetHubId: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Register a new `original` system token.
		OriginalSystemTokenRegistered { original: SystemTokenIdOf<T> },
		/// Deregister the `original` system token.
		OriginalSystemTokenDeregistered { original: SystemTokenIdOf<T> },
		/// Register a `wrapped` system token to an `original` system token.
		WrappedSystemTokenRegistered {
			original: SystemTokenIdOf<T>,
			para_id: SystemTokenOriginIdOf<T>,
		},
		/// Deregister a `wrapped` system token to an `original` system token.
		WrappedSystemTokenDeregistered {
			original: SystemTokenIdOf<T>,
			para_id: SystemTokenOriginIdOf<T>,
		},
		/// Update the fee rate of the parachain. The default value is 1_000.
		SetParaFeeRate { para_id: SystemTokenOriginIdOf<T>, para_fee_rate: SystemTokenBalanceOf<T> },
		/// Update the fee table of the parachain
		SetFeeTable {
			para_call_metadata:
				ParaCallMetadata<SystemTokenOriginIdOf<T>, SystemTokenPalletIdOf<T>>,
			fee: SystemTokenBalanceOf<T>,
		},
		/// Suspend a `original` system token.
		OriginalSystemTokenSuspended { original: SystemTokenIdOf<T> },
		/// Unsuspend the `original` system token.
		OriginalSystemTokenUnsuspended { original: SystemTokenIdOf<T> },
		/// Suspend a `wrapped` system token.
		WrappedSystemTokenSuspended {
			original: SystemTokenIdOf<T>,
			para_id: SystemTokenOriginIdOf<T>,
		},
		/// Unsuspend the `wrapped` system token.
		WrappedSystemTokenUnsuspended {
			original: SystemTokenIdOf<T>,
			para_id: SystemTokenOriginIdOf<T>,
		},
		/// Update exchange rates for given fiat currencies
		ExchangeRateUpdated { at: StandardUnixTime, updated: Vec<(Fiat, ExchangeRate)> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Requested `original` system token is already registered.
		OriginalAlreadyRegistered,
		/// Failed to remove the `original` system token as it is not registered.
		SystemTokenNotRegistered,
		/// Requested `wrapped` sytem token has already registered
		WrappedAlreadyRegistered,
		/// `Wrapped` system token has not been registered on Relay Chain
		WrappedNotRegistered,
		/// Registered System Tokens are out of limit
		TooManySystemTokensOnPara,
		/// Number of para ids using `original` system tokens has reached
		/// `MaxSystemTokenUsedParaIds`
		TooManyUsed,
		/// String metadata is out of limit
		BadMetadata,
		/// Deregister original's own wrapped token is not allowed
		BadAccess,
		/// Some of the value are stored on runtime(e.g key missing)
		NotFound,
		/// System tokens used by para id are not found
		ParaIdSystemTokensNotFound,
		/// Metadata of `original` system token is not found
		MetadataNotFound,
		/// Missing value of base system token weight
		WeightMissing,
		/// System token is already suspended
		AlreadySuspended,
		/// System token is not suspended
		NotSuspended,
		/// The paraid making the call is not the asset hub system parachain
		NotAssetHub,
		/// Pallet has not started yet
		NotInitiated,
		/// System Token has not been requested
		NotRequested,
		/// Exchange rate for given currency has not been requested
		ExchangeRateNotRequested,
		/// Error occurred on converting from `T::SystemTokenId` to SystemTokenId
		ErrorConvertToSystemTokenId,
		/// Error occurred while converting from 'original' to `wrapped`
		ErrorConvertToWrapped,
		/// Error occurred while converting to `RemoteAssetMetadata`
		ErrorConvertToRemoteAssetMetadata,
		/// Error occured while converting to some types
		ConversionError,
		/// Invalid System Token Weight(e.g `0`)
		InvalidSystemTokenWeight
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let l = UpdateExchangeRates::<T>::iter().count();
			// TODO: Find better way
			if l != 0 {
				let _ = UpdateExchangeRates::<T>::clear(u32::MAX, None);
				T::DbWeight::get().writes(l as u64)
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}

	/// Kind of fiat currencies needs to be requested. It is bounded for number of real-world
	/// currencies' types.
	///
	/// Flow
	///
	/// 1. Fiat will be stored when it is requested by enshrined runtime
	/// 2. Fiat stored on this list will be sent to runtime which implements `Oracle`
	/// 3. Oracle will send exchange rates for given fiat
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn request_fiat_list)]
	pub type RequestFiatList<T: Config> = StorageValue<_, Vec<Fiat>, ValueQuery>;

	/// Standard time for updating exchange rates
	#[pallet::storage]
	pub type RequestStandardTime<T: Config> = StorageValue<_, StandardUnixTime, ValueQuery>;

	/// Exchange rates for currencies relative to the base currency.
	#[pallet::storage]
	pub type ExchangeRates<T: Config> = StorageMap<_, Twox64Concat, Fiat, ExchangeRate>;

	#[pallet::storage]
	#[pallet::getter(fn system_token)]
	/// **Description:**
	///
	/// Properties of system token that stored useful data. Return `None` when there is no value.
	///
	/// **Key:**
	///
	/// `Original` System Token
	///
	/// **Value:**
	///
	/// `SystemTokenDetail`
	pub type SystemToken<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		SystemTokenIdOf<T>,
		SystemTokenDetail<
			SystemTokenWeightOf<T>,
			SystemTokenOriginIdOf<T>,
			T::MaxOriginalUsedParaIds,
		>,
	>;

	/// Updated exchange rates for `para_id` based on updated exchange rate data from Oracle
	#[pallet::storage]
	#[pallet::unbounded]
	pub type UpdateExchangeRates<T: Config> = StorageMap<
		_,
		Twox64Concat,
		SystemTokenOriginIdOf<T>,
		Vec<(SystemTokenIdOf<T>, SystemTokenWeightOf<T>)>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn original_system_token_metadata)]
	/// **Description:**
	///
	/// Metadata(`SystemTokenMetadata`, `AssetMetadata`) of `original` system token.
	/// Return `None` when there is no value.
	///
	/// **Key:**
	///
	/// `original` system token id
	///
	/// **Value:**
	///
	/// `SystemTokenMetadata`
	pub type Metadata<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		SystemTokenIdOf<T>,
		SystemTokenMetadata<SystemTokenBalanceOf<T>, BoundedStringOf<T>, BlockNumberFor<T>>,
	>;

	/// **Description:**
	///
	/// Map between `Fiat` and `Original` with `Wrapped` system token
	///
	/// **Key:**
	///
	/// `Fiat`
	///
	/// **Value:**
	///
	/// Vec of 'Original' SystemTokenId
	#[pallet::storage]
	#[pallet::unbounded]
	pub type FiatForOriginal<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Fiat,
		Twox64Concat,
		SystemTokenIdOf<T>, // Original
		BoundedVec<SystemTokenOriginIdOf<T>, T::MaxOriginalUsedParaIds>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn para_id_system_tokens)]
	/// **Description:**
	///
	/// List of system tokens(either `original` or `wrapped`) that are used from a parachain, which
	/// is identified by `para_id`
	///
	/// **Key:**
	///
	/// para_id
	///
	/// **Value:**
	///
	/// BoundedVec of either `original` or `wrapped` system token with maximum `MaxSystemTokens`
	pub type ParaIdSystemTokens<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		SystemTokenOriginIdOf<T>,
		BoundedVec<SystemTokenIdOf<T>, T::MaxSystemTokens>,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// **Description**:
		// Register (Original/Wrapped) SystemToken based on `SystemTokenType`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: `Original` system token id expected to be registered
		// - system_token_type: Register as `Original` or `Wrapped`
		// - extended_metadata: Additional metadata for `Original` System Token
		//
		// Process:
		// - Create & register `Original` System Token
		// - Create `wrapped` for Relay if None. Otherwise, create asset remotely via DMP
		#[pallet::call_index(0)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType<SystemTokenIdOf<T>, SystemTokenOriginIdOf<T>>,
			extended_metadata: Option<ExtendedMetadata>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let (original, maybe_para_id) = match system_token_type {
				SystemTokenType::Original(system_token_id) => {
					Self::do_register_system_token(&system_token_id, extended_metadata)?;
					(system_token_id, None)
				},
				SystemTokenType::Wrapped { original, para_id } => {
					ensure!(extended_metadata.is_none(), Error::<T>::BadAccess);
					(original, Some(para_id))
				},
			};
			Self::do_register_wrapped(&original, maybe_para_id)?;
			Self::deposit_event(Event::<T>::OriginalSystemTokenRegistered { original });

			Ok(())
		}

		#[pallet::call_index(1)]
		// Description:
		// Deregister SystemToken based on given `DeregisterKind`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: Original system token id expected to be deregistered
		// - kind: How should deregister work
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			kind: DeregisterKind<SystemTokenIdOf<T>, SystemTokenOriginIdOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::do_deregister_system_token(kind)?;
			Ok(())
		}

		#[pallet::call_index(2)]
		// Description:
		// Suspend all `original` and `wrapped` system token registered on runtime.
		// Suspended system token is no longer used as `transaction fee`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: Original system token id expected to be suspended
		pub fn suspend_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType<SystemTokenIdOf<T>, SystemTokenOriginIdOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			match system_token_type {
				SystemTokenType::Original(system_token_id) => {
					Self::suspend_all(&system_token_id)?;
					Self::deposit_event(Event::<T>::OriginalSystemTokenSuspended {
						original: system_token_id,
					});
				},
				SystemTokenType::Wrapped { original, para_id } => {
					Self::suspend(&original, &para_id, false)?;
					Self::deposit_event(Event::<T>::WrappedSystemTokenSuspended {
						original,
						para_id,
					});
				},
			}

			Ok(())
		}

		#[pallet::call_index(3)]
		// Description:
		// Unsuspend all `original` and `wrapped` system token registered on runtime.
		// Unsuspended system token is no longer used as `transaction fee`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: Original system token id expected to be unsuspended
		pub fn unsuspend_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType<SystemTokenIdOf<T>, SystemTokenOriginIdOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;
			match system_token_type {
				SystemTokenType::Original(system_token_id) => {
					Self::unsuspend_all(&system_token_id)?;
					Self::deposit_event(Event::<T>::OriginalSystemTokenUnsuspended {
						original: system_token_id,
					});
				},
				SystemTokenType::Wrapped { original, para_id } => {
					Self::unsuspend(&original, &para_id, false)?;
					Self::deposit_event(Event::<T>::WrappedSystemTokenUnsuspended {
						original,
						para_id,
					});
				},
			}

			Ok(())
		}

		#[pallet::call_index(4)]
		pub fn update_exchange_rate(
			origin: OriginFor<T>,
			standard_unix_time: StandardUnixTime,
			exchange_rates: Vec<(Fiat, ExchangeRate)>,
		) -> DispatchResult {
			Self::ensure_root_or_para(origin, <T as Config>::AssetHubId::get().into())?;
			Self::do_update_exchange_rate(standard_unix_time, exchange_rates)?;

			Ok(())
		}
	}
}

// System token related interal methods
impl<T: Config> Pallet<T>
where
	SystemTokenWeightOf<T>: From<F64>,
{
	/// Extend system token metadata for this runtime
	pub fn extend_metadata(
		metadata: &mut SystemTokenMetadata<
			SystemTokenBalanceOf<T>,
			BoundedStringOf<T>,
			BlockNumberFor<T>,
		>,
		extended: Option<ExtendedMetadata>,
	) -> Result<(), DispatchError> {
		if let Some(extended) = extended {
			let ExtendedMetadata { issuer, description, url } = extended;
			let bounded_issuer = Self::bounded_metadata(issuer);
			let bounded_description = Self::bounded_metadata(description);
			let bounded_url = Self::bounded_metadata(url);
			metadata.additional(bounded_issuer, bounded_description, bounded_url);
		}

		Ok(())
	}

	/// Bound some metadata info to `BoundedStringOf`
	fn bounded_metadata(
		byte: Vec<u8>,
	) -> BoundedStringOf<T> {
		if let Ok(bounded) = byte.try_into().map_err(|_| Error::<T>::BadMetadata) {
			return bounded
		} 
		Default::default()
	}

	/// Iterator of System Token for given currency
	fn fiat_for_originals(currency: &Fiat) -> KeyPrefixIterator<SystemTokenIdOf<T>> {
		FiatForOriginal::<T>::iter_key_prefix(currency)
	}

	fn do_update_system_token_weight(currency: &Fiat) -> DispatchResult {
		let os = Self::fiat_for_originals(currency);
		let mut k_v: Vec<(SystemTokenOriginIdOf<T>, SystemTokenIdOf<T>)> = Default::default();
		for o in os {
			let mut system_token_detail =
				SystemToken::<T>::get(&o).ok_or(Error::<T>::SystemTokenNotRegistered)?;
			let updated_sys_weight = Self::calc_system_token_weight(currency, &o)?;
			system_token_detail.update_weight(updated_sys_weight);
			let (origin_id, _, _) = o.id().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
			if let Some(para_id) = origin_id {
				k_v.push((para_id, o.clone()))
			}
			if let Some(ws) = FiatForOriginal::<T>::get(currency, &o) {
				for w in ws {
					k_v.push((w, o.clone()));
				}
			}
			for (para_id, original) in k_v {
				// TODO: Update for Relay Chain
				UpdateExchangeRates::<T>::try_mutate(para_id, |maybe_updated| -> DispatchResult {
					let mut updated = maybe_updated.take().unwrap_or_default();
					let wrapped =
						original.wrapped().map_err(|_| Error::<T>::ErrorConvertToWrapped)?;
					updated = vec![(wrapped, updated_sys_weight)];
					*maybe_updated = Some(updated);
					Ok(())
				})?;
			}
			k_v = Default::default();
		}
		Ok(())
	}

	/// Calculate `original` system token weight based on `FORMULA`
	///
	/// `FORMULA` = `BASE_WEIGHT` * `DECIMAL_RELATIVE_TO_BASE` / `EXCHANGE_RATE_RELATIVE_TO_BASE`
	fn calc_system_token_weight(
		currency: &Fiat,
		original: &SystemTokenIdOf<T>,
	) -> Result<SystemTokenWeightOf<T>, DispatchError> {
		let SystemConfig {base_system_token_detail, .. } = configuration::Pallet::<T>::active_system_token_config();
		let BaseSystemTokenDetail { base_currency, base_weight, base_decimals } = base_system_token_detail;
		let SystemTokenMetadata { decimals, .. } =
			Metadata::<T>::get(original).ok_or(Error::<T>::MetadataNotFound)?;
		let exponents: i32 = (base_decimals as i32) - (decimals as i32);
		let decimal_to_base: F64 = F64::from_i32(10).powi(exponents);
		let exchange_rate_to_base: F64 = if *currency != base_currency {
			ExchangeRates::<T>::get(currency)
				.ok_or(Error::<T>::ExchangeRateNotRequested)?
				.into()
		} else {
			F64::from_i32(1)
		};
		let f64_base_weight: F64 = F64::from_i128(base_weight as i128);
		let system_token_weight: SystemTokenWeightOf<T> =
			f64_base_weight.mul(decimal_to_base).div(exchange_rate_to_base).into();
		Ok(system_token_weight)
	}

	/// Check runtime origin for given `outer` origin. This only allows for `Root `or specific
	/// `para_id` origin
	fn ensure_root_or_para(
		origin: <T as frame_system::Config>::RuntimeOrigin,
		id: ParaId,
	) -> DispatchResult {
		if let Ok(para_id) = ensure_parachain(<T as Config>::RuntimeOrigin::from(origin.clone())) {
			// Check if matching para id...
			ensure!(para_id == id, Error::<T>::NotAssetHub);
		} else {
			// Check if root...
			ensure_root(origin.clone())?;
		}
		Ok(())
	}

	/// Update exchange rates which are from `Oracle`.
	///
	/// **Description:**
	///
	/// - Do nothing if the request fiat list is empty.
	/// - Otherwise, update the exchange rates for given `(fiat, rate)`
	///
	/// **Important:**
	///
	/// - Since exchange rates for given `Fiat` has been changed, all of runtimes taht used that
	///   system token should be updated.
	/// - Get all of the runtime info(e.g para_id) that used the system token.
	/// - Then store on `UpdateExchangeRates` with the key `para_id` and value **updated**
	///   `SystemTokenWeight`
	fn do_update_exchange_rate(
		at: StandardUnixTime,
		exchange_rates: Vec<(Fiat, ExchangeRate)>,
	) -> Result<(), DispatchError> {
		let request_fiat_list = Self::request_fiat_list();
		if request_fiat_list.len() == 0 {
			return Ok(())
		}
		RequestStandardTime::<T>::put(at);
		let mut updated_currency: Vec<(Fiat, ExchangeRate)> = Default::default();
		for (currency, rate) in exchange_rates.into_iter() {
			// Just in-case, check if the currency is requested
			if !request_fiat_list.contains(&currency) {
				continue
			}
			ExchangeRates::<T>::insert(&currency, &rate);
			Self::do_update_system_token_weight(&currency)?;
			updated_currency.push((currency, rate));
		}
		Self::deposit_event(Event::<T>::ExchangeRateUpdated { at, updated: updated_currency });
		Ok(())
	}

	/// **Description:**
	///
	/// Try get list of `wrapped` system tokens which is mapped to `original`
	///
	/// **Validity**
	///
	/// Ensure `original` system token is already registered
	fn list_all_para_ids_for(
		original: &SystemTokenIdOf<T>,
	) -> Result<Vec<SystemTokenOriginIdOf<T>>, Error<T>> {
		let system_token_detail =
			SystemToken::<T>::get(original).ok_or(Error::<T>::SystemTokenNotRegistered)?;
		Ok(system_token_detail.list_all_wrapped())
	}

	/// **Description:**
	///
	/// Process
	/// 1. Extend `SystemTokenMetadata` if any
	/// 2. Calculate `SystemTokenWeight` based on the type of currency
	/// 3. Send DMP if it is from parachain. Do it locally if it is from Relay Chain
	fn do_register_system_token(
		original: &SystemTokenIdOf<T>,
		extended_metadata: Option<ExtendedMetadata>,
	) -> DispatchResult {
		let now = frame_system::Pallet::<T>::block_number();
		let mut system_token_metadata =
			Metadata::<T>::get(&original).ok_or(Error::<T>::NotRequested)?;
		system_token_metadata.set_registered_at(now);
		let currency_type = system_token_metadata.currency_type();
		let system_token_weight = Self::calc_system_token_weight(&currency_type, original)?;
		Self::extend_metadata(&mut system_token_metadata, extended_metadata)?;
		let (origin_id, _, _) = original.id().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
		// Put `original` System Token for `Some(para_id)`
		// For Relay Chain, we insepct from `Fungibles`
		if let Some(para_id) = origin_id {
			Self::system_token_used_para_id(&para_id, &original)?;
		}
		// TODO
		// - Register Original from Relay Chain
		Self::try_register_system_token(&original, Some(system_token_weight.clone()))?;

		Metadata::<T>::insert(&original, system_token_metadata);
		SystemToken::<T>::insert(&original, SystemTokenDetail::new(system_token_weight));
		Ok(())
	}

	/// **Description:**
	///
	/// Try register `wrapped_system_token` and return `weight` of system token
	///
	/// **Validity:**
	///
	/// - `SystemTokenId` is already registered.
	///
	/// - `WrappedSystemTokenId` is not registered yet.
	///
	/// - `ParaId` is not registered yet.
	///
	/// **Changes:**
	///
	/// - `OriginalSystemTokenConverter`, `ParaIdSystemTokens`, `SystemTokenUsedParaIds`,
	///   `SystemTokenProperties`
	fn do_register_wrapped(
		original: &SystemTokenIdOf<T>,
		maybe_para_id: Option<SystemTokenOriginIdOf<T>>,
	) -> Result<(), DispatchError> {
		let mut system_token_detail =
			SystemToken::<T>::get(original).ok_or(Error::<T>::SystemTokenNotRegistered)?;
		let system_token_weight = system_token_detail.weight();
		ensure!(system_token_weight.ne(&Zero::zero()), Error::<T>::InvalidSystemTokenWeight);
		let system_token_metadata =
			Metadata::<T>::get(original).ok_or(Error::<T>::SystemTokenNotRegistered)?;
		if let Some(para_id) = maybe_para_id {
			ensure!(
				!system_token_detail.is_used_by(&para_id),
				Error::<T>::WrappedAlreadyRegistered
			);
			// Send DMP
			let wrapped_system_token_id =
				original.wrapped().map_err(|_| Error::<T>::ErrorConvertToWrapped)?;
			Self::try_create_wrapped(&para_id, &wrapped_system_token_id, system_token_weight)?;
			system_token_detail.register_wrapped_for(&para_id, SystemTokenStatus::Active)?;
			Self::system_token_used_para_id(&para_id, original)?;
			FiatForOriginal::<T>::try_mutate(
				&system_token_metadata.currency_type,
				&original,
				|maybe_para_ids| -> DispatchResult {
					// Since `Vec<_>` has default, it is safe to unwrap
					let mut para_ids = maybe_para_ids.take().unwrap_or_default();
					para_ids.try_push(para_id.clone()).map_err(|_| Error::<T>::TooManyUsed)?;
					*maybe_para_ids = Some(para_ids);
					Ok(())
				},
			)?;
			Self::deposit_event(Event::<T>::WrappedSystemTokenRegistered {
				original: original.clone(),
				para_id,
			})
		} else {
			// TODO: Relay Chain
			// T::Fungibles::touch(original)
		}
		Ok(())
	}

	/// **Description:**
	///
	/// Try suspend for all `original` and `wrapped` system tokens registered on runtime.
	///
	/// **Changes:**
	fn suspend_all(original: &SystemTokenIdOf<T>) -> DispatchResult {
		let para_ids = Self::list_all_para_ids_for(original)?;
		let wrapped = original.wrapped().map_err(|_| Error::<T>::ErrorConvertToWrapped)?;
		for para_id in para_ids {
			Self::suspend(&wrapped, &para_id, true)?;
		}

		Ok(())
	}

	/// **Description:**
	///
	/// Suspend for given `wrapped` system token.
	///
	/// **Changes:**
	///
	/// `SystemTokenProperties`
	fn suspend(
		original: &SystemTokenIdOf<T>,
		para_id: &SystemTokenOriginIdOf<T>,
		is_allowed_to_suspend_original: bool,
	) -> DispatchResult {
		// impl me!
		Ok(())
	}

	/// **Description:**
	///
	/// Try unsuspend for all `original` and `wrapped` system tokens registered on runtime.
	///
	/// **Changes:**
	fn unsuspend_all(original: &SystemTokenIdOf<T>) -> DispatchResult {
		let para_ids = Self::list_all_para_ids_for(original)?;
		let wrapped = original.wrapped().map_err(|_| Error::<T>::ErrorConvertToWrapped)?;
		for para_id in para_ids {
			Self::unsuspend(&wrapped, &para_id, true)?;
		}
		Ok(())
	}

	/// **Description:**
	///
	/// Unsuspend for given `wrapped` system token.
	///
	/// **Changes:**
	///
	/// `SystemTokenProperties`
	fn unsuspend(
		wrapped: &SystemTokenIdOf<T>,
		para_id: &SystemTokenOriginIdOf<T>,
		is_allowed_to_unsuspend_original: bool,
	) -> DispatchResult {
		// impl me!
		Ok(())
	}

	/// **Description:**
	///
	/// Try push `Original` System Token for `para_id` that are using System Token
	///
	/// **Errors:**
	///
	/// - `TooManySystemTokensOnPara`: Maximum number of elements has been reached for BoundedVec
	fn system_token_used_para_id(
		para_id: &SystemTokenOriginIdOf<T>,
		original: &SystemTokenIdOf<T>,
	) -> DispatchResult {
		ParaIdSystemTokens::<T>::try_mutate_exists(
			para_id.clone(),
			|maybe_used_system_tokens| -> Result<(), DispatchError> {
				let mut system_tokens = maybe_used_system_tokens
					.take()
					.map_or(Default::default(), |sys_tokens| sys_tokens);
				system_tokens
					.try_push(original.clone())
					.map_err(|_| Error::<T>::TooManySystemTokensOnPara)?;
				*maybe_used_system_tokens = Some(system_tokens);
				Ok(())
			},
		)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try remove `ParaIdSystemTokens` for any(`original` or `wrapped`) system token id
	fn remove_system_token_for_para_id(
		system_token_id: &SystemTokenIdOf<T>,
		para_id: &SystemTokenOriginIdOf<T>,
	) -> DispatchResult {
		ParaIdSystemTokens::<T>::try_mutate_exists(
			para_id,
			|maybe_system_tokens| -> Result<(), DispatchError> {
				let mut system_tokens =
					maybe_system_tokens.take().ok_or(Error::<T>::ParaIdSystemTokensNotFound)?;
				system_tokens.retain(|x| x != system_token_id);
				if system_tokens.is_empty() {
					*maybe_system_tokens = None;
					return Ok(())
				}
				*maybe_system_tokens = Some(system_tokens);
				Ok(())
			},
		)?;

		Ok(())
	}
}

// XCM-related internal methods
impl<T: Config> Pallet<T> 
where
	DestIdOf<T>: From<SystemTokenOriginIdOf<T>>
{
	/// **Description:**
	///
	/// Deregister system token for given `kind`
	fn do_deregister_system_token(
		kind: DeregisterKind<SystemTokenIdOf<T>, SystemTokenOriginIdOf<T>>,
	) -> DispatchResult {
		match kind {
			DeregisterKind::All(original) => {
				let system_token_detail =
					SystemToken::<T>::get(&original).ok_or(Error::<T>::SystemTokenNotRegistered)?;
				let (origin_id, _, _) =
					original.id().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
				for para_id in system_token_detail.list_all_wrapped().iter() {
					Self::remove_system_token_for_para_id(&original, para_id)?;
				}
				SystemToken::<T>::remove(&original);
				Metadata::<T>::remove(&original);
				if let Some(para_id) = origin_id {
					T::SystemTokenHandler::deregister_system_token(para_id.into(), original);
				} else {
					// TODO: Relay Chain
					// Fungibles::deregister()
				}
			},
			DeregisterKind::Specific { original, wrapped } => {
				if let Some(para_id) = wrapped {
					let system_token_detail = SystemToken::<T>::get(&original)
						.ok_or(Error::<T>::SystemTokenNotRegistered)?;
					ensure!(
						system_token_detail.is_used_by(&para_id),
						Error::<T>::WrappedNotRegistered
					);
					Self::remove_system_token_for_para_id(&original, &para_id)?;
				} else {
					// TODO: Relay Chain
					// Fungibles::deregister()
				}
			},
		}
		Ok(())
	}

	/// **Description:**
	///
	/// Try sending DMP of call `promote` to specific parachain.
	/// If success, destination parachain's local asset's `sufficient` state to `is_sufficient`, and
	/// set its weight
	fn try_register_system_token(
		system_token_id: &SystemTokenIdOf<T>,
		system_token_weight: Option<SystemTokenWeightOf<T>>,
	) -> DispatchResult {
		let (origin_id, _, _) =
			system_token_id.id().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
		let weight = system_token_weight.ok_or(Error::<T>::WeightMissing)?;
		if let Some(para_id) = origin_id {
			T::SystemTokenHandler::register_system_token(para_id.into(), system_token_id.clone(), weight);
		} else {
			// TODO: Relay Chain
			// T::Fungibles::register()
		}
		Ok(())
	}

	/// **Description:**
	///
	/// Try create `wrapped` system token to local
	///
	/// **Params:**
	///
	/// - wrapped: `wrapped` system token expected to be created
	///
	/// - system_token_weight: Weight of system token to store on local asset
	///
	/// **Logic:**
	///
	/// If `para_id == 0`, call internal `Assets` pallet method.
	/// Otherwise, send DMP of `force_create_with_metadata` to expected `para_id` destination
	fn try_create_wrapped(
		para_id: &SystemTokenOriginIdOf<T>,
		original: &SystemTokenIdOf<T>,
		system_token_weight: SystemTokenWeightOf<T>,
	) -> DispatchResult {
		let original_metadata =
			Metadata::<T>::get(original).ok_or(Error::<T>::MetadataNotFound)?;
		T::SystemTokenHandler::create_wrapped(
			para_id.clone().into(),
			original.clone(),
			original_metadata.currency_type,
			original_metadata.min_balance,
			original_metadata.name.to_vec(),
			original_metadata.symbol.to_vec(),
			original_metadata.decimals,
			system_token_weight,
		);
		Ok(())
	}

	pub fn requested_asset_metadata(bytes: &mut Vec<u8>) {
		if let Ok(remote_asset_metadata) = RemoteAssetMetadata::<
				SystemTokenIdOf<T>,
				SystemTokenBalanceOf<T>,
			>::decode(&mut &bytes[..]) {
			let RemoteAssetMetadata {
				asset_id,
				name,
				symbol,
				currency_type,
				decimals,
				min_balance,
			} = remote_asset_metadata;
			if let Ok((origin_id, pallet_id, asset_id)) = asset_id.id()
			{
				let system_token_id =
					T::SystemTokenId::convert_back(origin_id, pallet_id, asset_id);
				Metadata::<T>::insert(
					system_token_id,
					SystemTokenMetadata::new(
						currency_type.clone(),
						name,
						symbol,
						decimals,
						min_balance,
					),
				);
				RequestFiatList::<T>::mutate(|request_fiat| {
					if !request_fiat.contains(&currency_type) {
						request_fiat.push(currency_type);
					}
				});
			} else {
				log::error!("❌ Failed to convert to SystemTokenId ❌");
				return
			}
		} else {
			log::error!("❌ Failed to decode RemoteAssetMetadata ❌");
			return
		}
	}
}

pub mod types {

	use super::*;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	pub type SystemTokenIdOf<T> = <T as Config>::SystemTokenId;
	pub type SystemTokenAssetIdOf<T> = <<T as Config>::SystemTokenId as SystemTokenId>::AssetId;
	pub type SystemTokenOriginIdOf<T> = <<T as Config>::SystemTokenId as SystemTokenId>::OriginId;
	pub type SystemTokenPalletIdOf<T> = <<T as Config>::SystemTokenId as SystemTokenId>::PalletId;
	pub type SystemTokenWeightOf<T> = <<T as Config>::Fungibles as InspectSystemToken<
		<T as frame_system::Config>::AccountId,
	>>::SystemTokenWeight;
	pub type SystemTokenBalanceOf<T> =
		<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub type BoundedStringOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;
	pub type DestIdOf<T> = <<T as Config>::SystemTokenHandler as SystemTokenInterface>::DestId;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum SystemTokenType<SystemTokenId, ParaId> {
		Original(SystemTokenId),
		Wrapped { original: SystemTokenId, para_id: ParaId },
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub enum SystemTokenStatus {
		Active,
		Suspended,
		#[default]
		Pending,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum DeregisterKind<SystemTokenId, ParaId> {
		/// Deregister all related to `SystemTokenIdOf<T>`
		All(SystemTokenId),
		/// Deregister for specific `Option<ParaId>`. If `None`, it means `Relay Chain`
		Specific { original: SystemTokenId, wrapped: Option<ParaId> },
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ParaCallMetadata<ParaId, PalletId> {
		pub(crate) para_id: ParaId,
		pub(crate) pallet_id: PalletId,
		pub(crate) pallet_name: Vec<u8>,
		pub(crate) call_name: Vec<u8>,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ExtendedMetadata {
		/// The user friendly name of issuer in real world
		pub(crate) issuer: Vec<u8>,
		/// The description of the token
		pub(crate) description: Vec<u8>,
		/// The url of related to the token or issuer
		pub(crate) url: Vec<u8>,
	}

	/// Detail for `Original` System Token
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(MaxUsed))]
	pub struct SystemTokenDetail<Weight, ParaId, MaxUsed: Get<u32>> {
		/// Weight of System Token for adjusting weight transaction vote
		pub(crate) system_token_weight: Weight,
		/// Status of System Token
		pub(crate) system_token_status: SystemTokenStatus,
		/// List of para_ids that are using this System Token
		pub(crate) para_ids: BoundedVec<(ParaId, SystemTokenStatus), MaxUsed>,
	}

	impl<Weight: Clone, ParaId: PartialEq + Clone, MaxUsed: Get<u32>> SystemTokenDetail<Weight, ParaId, MaxUsed> {
		pub fn new(system_token_weight: Weight) -> Self {
			Self {
				system_token_weight,
				system_token_status: SystemTokenStatus::Active,
				para_ids: BoundedVec::new(),
			}
		}

		pub fn weight(&self) -> Weight {
			self.system_token_weight.clone()
		}

		/// Check if given `para_id` is using wrapped
		pub fn is_used_by(&self, para_id: &ParaId) -> bool {
			self.para_ids.iter().any(|(p, _)| p == para_id)
		}

		/// Try push `para_id` to `wrapped` list
		pub fn register_wrapped_for(
			&mut self,
			para_id: &ParaId,
			system_token_status: SystemTokenStatus,
		) -> DispatchResult {
			if let Err(_) = self.para_ids.try_push((para_id.clone(), system_token_status)) {
				// TODO
				Ok(())
			} else {
				Ok(())
			}
		}

		/// List all `para_ids` which are `SystemTokenStatus::Active` that are using this System Token
		pub fn list_all_wrapped(&self) -> Vec<ParaId> {
			self.para_ids.clone()
				.into_iter()
				.filter_map(|(p, s)| if s == SystemTokenStatus::Active { Some(p) } else { None })
				.collect()
		}

		pub fn update_weight(&mut self, new: Weight) {
			self.system_token_weight = new;
		}
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo
	)]
	/// Metadata of the `original` asset from enshrined runtime
	pub struct SystemTokenMetadata<Balance, BoundedString, BlockNumber> {
		pub(crate) currency_type: Fiat,
		/// The user friendly name of this system token.
		pub(crate) name: Vec<u8>,
		/// The exchange symbol for this system token.
		pub(crate) symbol: Vec<u8>,
		/// The number of decimals this asset uses to represent one unit.
		pub(crate) decimals: u8,
		/// The minimum balance of this new asset that any single account must
		/// have. If an account's balance is reduced below this, then it collapses to zero.
		#[codec(compact)]
		pub(crate) min_balance: Balance,
		/// The time of when system token registered
		#[codec(compact)]
		pub(crate) registered_at: BlockNumber,
		/// The user friendly name of issuer in real world
		pub(crate) issuer: BoundedString,
		/// The description of the token
		pub(crate) description: BoundedString,
		/// The url of related to the token or issuer
		pub(crate) url: BoundedString,
	}

	impl<Balance: Default, BoundedString: Default, BlockNumber: Default>
		SystemTokenMetadata<Balance, BoundedString, BlockNumber>
	{
		pub fn currency_type(&self) -> Fiat {
			self.currency_type.clone()
		}

		pub fn set_registered_at(&mut self, at: BlockNumber) {
			self.registered_at = at;
		}

		pub fn new(
			currency_type: Fiat,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			min_balance: Balance,
		) -> Self {
			Self { currency_type, name, symbol, decimals, min_balance, ..Default::default() }
		}

		pub fn additional(
			&mut self,
			issuer: BoundedString,
			description: BoundedString,
			url: BoundedString,
		) {
			self.issuer = issuer;
			self.description = description;
			self.url = url;
		}
	}
}

pub mod traits {
	
	use super::*;
	use frame_support::Parameter;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	/// API for handling System Token related methods 
	/// Generally implemented by the Relay-chain
	pub trait SystemTokenInterface {

		/// Location for asset
		type Location: Parameter;
		/// Type of System Token balance
		type Balance: AtLeast32BitUnsigned;
		/// Type of System Token weight
		type SystemTokenWeight: AtLeast32BitUnsigned;
		/// Type of destination id(e.g para_id)
		type DestId: Parameter;
	
		/// Register `Original` System Token for `dest_id` Runtime(e.g `set_sufficient=true`)
		fn register_system_token(dest_id: Self::DestId, asset_id: Self::Location, system_token_weight: Self::SystemTokenWeight);
		/// Deregister `Original/Wrapped` System Token for `dest_id` Runtime
		fn deregister_system_token(dest_id: Self::DestId, asset_id: Self::Location);
		/// Create local asset of `Wrapped` System Token for `dest_id` Runtime
		fn create_wrapped(
			dest_id: Self::DestId,
			original: Self::Location,
			currency_type: Fiat,
			min_balance: Self::Balance,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			system_token_weight: Self::SystemTokenWeight,
		);
	}
}