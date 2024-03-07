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

use crate::{configuration, ensure_parachain, paras, Origin as ParachainOrigin, ParaId, SystemTokenInterface};
use frame_support::storage::KeyPrefixIterator;
pub use frame_support::{pallet_prelude::*, traits::UnixTime};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use softfloat::F64;
use sp_runtime::types::{infra_core::*, token::*, vote::*};
use sp_std::prelude::*;
use types::*;
use xcm::latest::{MultiLocation, Junctions, Junction};

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + configuration::Config + paras::Config {
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<ParachainOrigin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Id of System Token that used in this module
		type SystemTokenId: Into<MultiLocation> + TryFrom<MultiLocation>;
		/// Core interface for InfraBlockchain Runtime
		type InfraCore: UpdateInfraConfig<
				AssetId=SystemTokenAssetId,
				ParaId=SystemTokenParaId,
				SystemTokenWeight=SystemTokenWeight
			> 
			+ RuntimeConfigProvider 
			+ VotingHandler;
		/// Time used for computing registration date.
		type UnixTime: UnixTime;
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
		OriginalSystemTokenRegistered { original: SystemTokenId },
		/// Deregister the `original` system token.
		OriginalSystemTokenDeregistered { original: SystemTokenId },
		/// Register a `wrapped` system token to an `original` system token.
		WrappedSystemTokenRegistered { original: SystemTokenId, wrapped: SystemTokenId },
		/// Deregister a `wrapped` system token to an `original` system token.
		WrappedSystemTokenDeregistered { wrapped: SystemTokenId },
		/// Converting from `wrapped` to `original` has happened
		SystemTokenConverted { from: SystemTokenId, to: SystemTokenId },
		/// Update the weight for system token. The weight is calculated based on the exchange_rate
		/// and decimal.
		SetSystemTokenWeight { system_token_id: SystemTokenId, property: SystemTokenProperty },
		/// Update the fee rate of the parachain. The default value is 1_000.
		SetParaFeeRate { para_id: SystemTokenParaId, para_fee_rate: SystemTokenBalance },
		/// Update the fee table of the parachain
		SetFeeTable { para_call_metadata: ParaCallMetadata, fee: SystemTokenBalance },
		/// Suspend a `original` system token.
		OriginalSystemTokenSuspended { original: SystemTokenId },
		/// Unsuspend the `original` system token.
		OriginalSystemTokenUnsuspended { original: SystemTokenId },
		/// Suspend a `wrapped` system token.
		WrappedSystemTokenSuspended { wrapped: SystemTokenId },
		/// Unsuspend the `wrapped` system token.
		WrappedSystemTokenUnsuspended { wrapped: SystemTokenId },
		/// Update exchange rates for given fiat currencies
		ExchangeRateUpdated { at: StandardUnixTime, updated: Vec<(Fiat, ExchangeRate)> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Requested `original` system token is already registered.
		OriginalAlreadyRegistered,
		/// Failed to remove the `original` system token as it is not registered.
		OriginalNotRegistered,
		/// Requested `wrapped` sytem token has already registered
		WrappedAlreadyRegistered,
		/// `Wrapped` system token has not been registered on Relay Chain
		WrappedNotRegistered,
		/// `Wrapped` system token id is not provided when register `orinal` system token
		WrappedForRelayNotProvided,
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
		/// Property of system token is not found
		PropertyNotFound,
		/// System tokens used by para id are not found
		ParaIdSystemTokensNotFound,
		/// A specific system token used para ids are not found
		ParaIdsNotFound,
		/// Metadata of `original` system token is not found
		MetadataNotFound,
		/// Missing value of base system token weight
		WeightMissing,
		/// Error occurred on sending XCM
		DmpError,
		/// System token is already suspended
		AlreadySuspended,
		/// System token is not suspended
		NotSuspended,
		/// System token metadata is not provided when registered `original` system token
		SystemTokenMetadataNotProvided,
		/// Asset metadata is not provided when registered `original` system token
		AssetMetadataNotProvided,
		/// The paraid making the call is not the asset hub system parachain
		NotAssetHub,
		/// Pallet has not started yet
		NotInitiated,
		/// System Token has not been requested
		NotRequested,
		/// Exchange rate for given currency has not been requested
		ExchangeRateNotRequested,
		/// Invalid Id syntax for SystemToken
		InvalidSystemTokenId,
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

	/// Updated exchange rates for `para_id` based on updated exchange rate data from Oracle
	#[pallet::storage]
	#[pallet::unbounded]
	pub type UpdateExchangeRates<T: Config> = StorageMap<
		_,
		Twox64Concat,
		SystemTokenParaId,
		Vec<(SystemTokenAssetId, SystemTokenWeight)>,
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
	pub type OriginalSystemTokenMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, SystemTokenIdOf<T>, SystemTokenMetadata<BoundedStringOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn system_token_properties)]
	/// **Description:**
	///
	/// Properties of system token that stored useful data. Return `None` when there is no value.
	///
	/// **Key:**
	///
	/// (`original` or `wrapped`) system token id
	///
	/// **Value:**
	///
	/// `SystemTokenProperty`
	pub type SystemTokenProperties<T: Config> =
		StorageMap<_, Blake2_128Concat, SystemTokenIdOf<T>, SystemTokenProperty>;

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
		SystemTokenIdOf<T>,
		BoundedVec<SystemTokenIdOf<T>, T::MaxOriginalUsedParaIds>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn wrapped_system_token_on_para)]
	/// **Description:**
	///
	/// Used for converting `wrapped` system token to `original` system token
	///
	/// **Key:**
	///
	/// `wrapped` system token id
	///
	/// **Value:**
	///
	/// `original` system token id
	pub type OriginalSystemTokenConverter<T: Config> =
		StorageMap<_, Blake2_128Concat, InfraAssetIdOf<T>, InfraAssetIdOf<T>>;

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
		InfraParaIdOf<T>,
		BoundedVec<InfraAssetIdOf<T>, T::MaxSystemTokens>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn system_token_used_para_ids)]
	/// **Description:**
	///
	/// List of para_ids that are using a `original` system token, which is key
	///
	/// **Key:**
	///
	/// `original` system token id
	///
	/// **Value:**
	///
	/// BoundedVec of `para_id` with maximum `MaxSystemTokenUsedParaIds`
	pub type SystemTokenUsedParaIds<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		InfraAssetIdOf<T>,
		BoundedVec<InfraParaIdOf<T>, T::MaxOriginalUsedParaIds>,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Description:
		// Register system token after creating local asset on original chain.
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: `Original` system token id expected to be registered
		// - wrapped_fore_relay_chain: Original's `Wrapped` system token for relay chain
		// - issuer: Human-readable issuer of system token which is metadata for Relay Chain
		// - description: Human-readable description of system token which is metadata for Relay
		//   Chain
		// - url: Human-readable url of system token which is metadata for Relay Chain
		// - name: Name of system token which is same as original chain
		// - symbol: Symbol of system token which is same as original chain
		// - decimals: Decimal of system token which is same as original chain
		// - min_balance: Min balance of system token to be alive which is same as original chain
		// - system_token_weight: Weight of system token which is same as original chain
		//
		// Logic:
		// Once it is accepted by governance,
		// `try_register_original` and `try_promote` will be called,
		// which will change `original` system token state
		#[pallet::call_index(0)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType<SystemTokenIdOf<T>>,
			wrapped_for_relay_chain: Option<SystemTokenIdOf<T>>,
			extended_metadata: Option<ExtendedMetadata>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let (original, wrapped) = match system_token_type {
				SystemTokenType::Original(original) => {
					Self::try_register_original(&original, extended_metadata)?;
					let wrapped =
						wrapped_for_relay_chain.ok_or(Error::<T>::WrappedForRelayNotProvided)?;
					ensure!(wrapped.para_id == RELAY_CHAIN_PARA_ID, Error::<T>::BadAccess);
					(original, wrapped)
				},
				SystemTokenType::Wrapped { original, wrapped } => {
					ensure!(extended_metadata.is_none(), Error::<T>::BadAccess);
					(original, wrapped)
				},
			};
			// Create wrapped for Relay Chain
			let (system_token_weight, currency_type) =
				Self::try_register_wrapped(&original, &wrapped)?;
			Self::try_create_wrapped(&wrapped, system_token_weight)?;
			FiatForOriginal::<T>::try_mutate(
				&currency_type,
				&original,
				|maybe_wrapped| -> DispatchResult {
					// Since `Vec<_>` has default, it is safe to unwrap
					let mut wrapped_list = maybe_wrapped.take().unwrap_or_default();
					wrapped_list.try_push(wrapped).map_err(|_| Error::<T>::TooManyUsed)?;
					*maybe_wrapped = Some(wrapped_list);
					Ok(())
				},
			)?;
			Self::deposit_event(Event::<T>::OriginalSystemTokenRegistered { original });
			Self::deposit_event(Event::<T>::WrappedSystemTokenRegistered { original, wrapped });

			Ok(())
		}

		#[pallet::call_index(1)]
		// Description:
		// Deregister all `original` and `wrapped` system token registered on runtime.
		// Deregistered system token is no longer used as `transaction fee`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: Original system token id expected to be deregistered
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType<SystemTokenIdOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;
			match system_token_type {
				SystemTokenType::Original(original) => {
					Self::try_deregister_all(original)?;
					Self::deposit_event(Event::<T>::OriginalSystemTokenDeregistered { original });
				},
				SystemTokenType::Wrapped { wrapped, .. } => {
					Self::try_deregister(&wrapped, false)?;
					Self::deposit_event(Event::<T>::WrappedSystemTokenDeregistered { wrapped });
				},
			}

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
			system_token_type: SystemTokenType<SystemTokenIdOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			match system_token_type {
				SystemTokenType::Original(original) => {
					Self::try_suspend_all(original)?;
					Self::deposit_event(Event::<T>::OriginalSystemTokenSuspended { original });
				},
				SystemTokenType::Wrapped { wrapped, .. } => {
					Self::try_suspend(&wrapped, false)?;
					Self::deposit_event(Event::<T>::WrappedSystemTokenSuspended { wrapped });
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
			system_token_type: SystemTokenType,
		) -> DispatchResult {
			ensure_root(origin)?;
			match system_token_type {
				SystemTokenType::Original(original) => {
					Self::try_unsuspend_all(original)?;
					Self::deposit_event(Event::<T>::OriginalSystemTokenUnsuspended { original });
				},
				SystemTokenType::Wrapped { wrapped, .. } => {
					Self::try_unsuspend(&wrapped, false)?;
					Self::deposit_event(Event::<T>::WrappedSystemTokenUnsuspended { wrapped });
				},
			}

			Ok(())
		}

		#[pallet::call_index(4)]
		// Description:
		// Try send DMP for encoded `update_para_fee_rate` to given `para_id`
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - para_id: Destination of DMP
		// - pallet_id: Pallet index of `update_fee_para_rate`
		// - para_fee_rate: Fee rate for specific parachain expected to be updated
		pub fn update_para_fee_rate(
			origin: OriginFor<T>,
			para_id: SystemTokenParaId,
			para_fee_rate: SystemTokenBalance,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::InfraCore::update_para_fee_rate(para_id, para_fee_rate);
			Self::deposit_event(Event::<T>::SetParaFeeRate { para_id, para_fee_rate });

			Ok(())
		}

		#[pallet::call_index(5)]
		// Description:
		// Setting fee for parachain-specific calls(extrinsics).
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - para_id: Id of the parachain of which fee to be set
		// - pallet_id: Pallet index of `System Token`
		// - pallet_name: Name of the pallet of which extrinsic is defined
		// - call_name: Name of the call(extrinsic) of which fee to be set
		// - fee: Amount of fee to be set
		//
		// Logic:
		// When fee is charged on the parachain, extract the metadata of the call, which is
		// 'pallet_name' and 'call_name' Lookup the fee table with `key = (pallet_name, call_name)`.
		// If value exists, we charged with that fee. Otherwise, default fee will be charged
		pub fn update_fee_table(
			origin: OriginFor<T>,
			para_call_metadata: ParaCallMetadata,
			fee: SystemTokenBalance,
		) -> DispatchResult {
			ensure_root(origin)?;

			let ParaCallMetadata { para_id, pallet_name, call_name, .. } =
				para_call_metadata.clone();
			T::InfraCore::update_fee_table(para_id, pallet_name, call_name, fee);
			Self::deposit_event(Event::<T>::SetFeeTable { para_call_metadata, fee });

			Ok(())
		}

		#[pallet::call_index(6)]
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
impl<T: Config> Pallet<T> {
	fn unix_time() -> u128 {
		T::UnixTime::now().as_millis()
	}

	/// Extend system token metadata for this runtime
	pub fn extend_metadata(
		metadata: &mut SystemTokenMetadata<BoundedStringOf<T>>,
		extended: Option<ExtendedMetadata>,
	) -> Result<(), DispatchError> {
		if let Some(extended) = extended {
			let ExtendedMetadata { issuer, description, url } = extended;
			let bounded_issuer = Self::bounded_metadata(issuer)?;
			let bounded_description = Self::bounded_metadata(description)?;
			let bounded_url = Self::bounded_metadata(url)?;
			metadata.additional(bounded_issuer, bounded_description, bounded_url);
		}

		Ok(())
	}

	/// Bound some metadata info to `BoundedStringOf`
	fn bounded_metadata(
		byte: Option<Vec<u8>>,
	) -> Result<Option<BoundedStringOf<T>>, DispatchError> {
		if let Some(b) = byte {
			let bounded = b.try_into().map_err(|_| Error::<T>::BadMetadata)?;
			Ok(Some(bounded))
		} else {
			Ok(None)
		}
	}

	fn fiat_for_originals(currency: &Fiat) -> KeyPrefixIterator<SystemTokenId> {
		FiatForOriginal::<T>::iter_key_prefix(currency)
	}

	fn try_update_system_token_weight(currency: &Fiat) -> DispatchResult {
		let os = Self::fiat_for_originals(currency);
		let mut k_v: Vec<(SystemTokenParaId, SystemTokenAssetId)> = Default::default();
		for o in os {
			let updated_sys_weight = Self::calc_system_token_weight(currency, &o)?;
			Self::try_update_weight_property(&o, updated_sys_weight)?;
			let SystemTokenId { para_id, asset_id, .. } = o.clone();
			k_v.push((para_id, asset_id));
			if let Some(ws) = FiatForOriginal::<T>::get(currency, &o) {
				for w in ws {
					Self::try_update_weight_property(&w, updated_sys_weight)?;
					let SystemTokenId { para_id, asset_id, .. } = w;
					k_v.push((para_id, asset_id));
				}
			}
			for (para_id, asset_id) in k_v {
				if para_id == RELAY_CHAIN_PARA_ID {
					T::InfraCore::update_system_token_weight(asset_id, updated_sys_weight)
				} else {
					UpdateExchangeRates::<T>::try_mutate(
						para_id,
						|maybe_updated| -> DispatchResult {
							let mut updated = maybe_updated.take().unwrap_or_default();
							updated = vec![(asset_id, updated_sys_weight)];
							*maybe_updated = Some(updated);
							Ok(())
						},
					)?;
				}
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
		original: &SystemTokenId,
	) -> Result<SystemTokenWeight, DispatchError> {
		let BaseSystemTokenDetail { base_weight, base_decimals, base_currency } =
			T::InfraCore::infra_system_config()
				.map_err(|_| Error::<T>::NotInitiated)?
				.base_system_token_detail
				.clone();
		let SystemTokenMetadata { decimals, .. } =
			OriginalSystemTokenMetadata::<T>::get(original).ok_or(Error::<T>::MetadataNotFound)?;
		let exponents: i32 = (base_decimals as i32) - (decimals as i32);
		let decimal_to_base: F64 = F64::from_i32(10).powi(exponents);
		let exchange_rate_to_base: F64 = if *currency != base_currency {
			ExchangeRates::<T>::get(currency)
				.ok_or(Error::<T>::ExchangeRateNotRequested)?
				.into()
		} else {
			F64::from_i32(1)
		};
		let f64_base_weight: F64 = base_weight.into();
		let system_token_weight: u128 =
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
			Self::try_update_system_token_weight(&currency)?;
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
	fn try_get_wrapped_system_token_list(
		original: &SystemTokenId,
	) -> Result<Vec<SystemTokenId>, Error<T>> {
		ensure!(
			SystemTokenProperties::<T>::contains_key(&original),
			Error::<T>::OriginalNotRegistered
		);
		let token_list = OriginalSystemTokenConverter::<T>::iter_keys()
			.filter(|wrapped| {
				OriginalSystemTokenConverter::<T>::get(wrapped).as_ref() == Some(original)
			})
			.collect::<Vec<SystemTokenId>>();
		Ok(token_list)
	}

	/// **Description:**: SystemTokenMetadata<BoundedVec<u8, <T as Config>::StringLimit>>
	///
	/// Try register `original` system token on runtime.
	///
	/// **Validity:**
	///
	/// - Ensure `original` system token is not registered in `OriginalSystemTokenMetadata`
	///
	/// **Changes:**
	///
	/// `ParaIdSystemTokens`, `SystemTokenUsedParaIds`, `SystemTokenProperties`,
	/// `OriginalSystemTokenConverter`, `OriginalSystemTokenMetadata`
	fn try_register_original(
		original: &SystemTokenIdOf<T>,
		extended_metadata: Option<ExtendedMetadata>,
	) -> DispatchResult {
		let mut system_token_metadata =
			OriginalSystemTokenMetadata::<T>::get(&original).ok_or(Error::<T>::NotRequested)?;
		system_token_metadata.set_registered_at(Self::unix_time());
		let currency_type = system_token_metadata.currency_type();
		let system_token_weight = Self::calc_system_token_weight(&currency_type, original)?;
		Self::extend_metadata(&mut system_token_metadata, extended_metadata)?;
		let SystemTokenId { para_id, .. } = original.clone();
		Self::try_push_sys_token_for_para_id(para_id, &original)?;
		Self::try_push_para_id(para_id, &original)?;
		// TODO: What if register System Token on Relay Chain?
		Self::try_promote(&original, Some(system_token_weight))?;
		// TODO: Deprecate `SystemTokenProperties`
		SystemTokenProperties::<T>::insert(
			&original,
			SystemTokenProperty {
				system_token_weight: Some(system_token_weight),
				status: SystemTokenStatus::Active,
				created_at: Self::unix_time(),
			},
		);
		// Self registered as wrapped token, which would be used for knowing it is 'original system
		// token'
		OriginalSystemTokenConverter::<T>::insert(&original, &original);
		OriginalSystemTokenMetadata::<T>::insert(&original, system_token_metadata);

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
	fn try_register_wrapped(
		original: &SystemTokenId,
		wrapped: &SystemTokenId,
	) -> Result<(SystemTokenWeight, Fiat), DispatchError> {
		ensure!(
			!OriginalSystemTokenConverter::<T>::contains_key(wrapped),
			Error::<T>::WrappedAlreadyRegistered
		);
		let system_token_metadata = OriginalSystemTokenMetadata::<T>::get(original)
			.ok_or(Error::<T>::OriginalNotRegistered)?;
		let SystemTokenId { para_id, .. } = wrapped.clone();

		let para_ids = SystemTokenUsedParaIds::<T>::get(original)
			.map_or(Default::default(), |para_id| para_id);
		ensure!(!para_ids.contains(&para_id), Error::<T>::WrappedAlreadyRegistered);

		OriginalSystemTokenConverter::<T>::insert(wrapped, original);
		SystemTokenProperties::<T>::insert(
			wrapped,
			&SystemTokenProperty {
				system_token_weight: None,
				status: SystemTokenStatus::Active,
				created_at: Self::unix_time(),
			},
		);

		Self::try_push_sys_token_for_para_id(para_id, wrapped)?;
		Self::try_push_para_id(para_id, original)?;

		let property =
			SystemTokenProperties::<T>::get(original).ok_or(Error::<T>::PropertyNotFound)?;
		let weight = property.system_token_weight.ok_or(Error::<T>::WeightMissing)?;
		Ok((weight, system_token_metadata.currency_type()))
	}

	/// **Description:**
	///
	/// Try deregister for all `original` and `wrapped` system tokens registered on runtime.
	///
	/// **Changes:**
	fn try_deregister_all(original: SystemTokenId) -> DispatchResult {
		let wrapped_system_tokens = Self::try_get_wrapped_system_token_list(&original)?;
		for wrapped_system_token_id in wrapped_system_tokens {
			Self::try_deregister(&wrapped_system_token_id, true)?;
		}

		Ok(())
	}

	/// **Description:**
	///
	/// Remove value for given `wrapped` system token.
	///
	/// **Changes:**
	///
	/// `ParaIdSystemTokens`, `SystemTokenUsedParaIds`, `OriginalSystemTokenConverter`,
	/// `SystemTokenProperties`
	fn try_deregister(
		wrapped: &SystemTokenId,
		is_allowed_to_remove_original: bool,
	) -> DispatchResult {
		let original = OriginalSystemTokenConverter::<T>::get(&wrapped)
			.ok_or(Error::<T>::WrappedNotRegistered)?;

		let SystemTokenId { para_id, .. } = wrapped.clone();
		if original.para_id == para_id && !is_allowed_to_remove_original {
			return Err(Error::<T>::BadAccess.into())
		}

		// Case: Original's self-wrapped
		let (system_token_id, is_unlink) = if original.para_id == para_id {
			OriginalSystemTokenMetadata::<T>::remove(original);
			(original, false)
		} else {
			(wrapped.clone(), true)
		};
		Self::try_demote(system_token_id, is_unlink)?;
		Self::try_remove_sys_token_for_para(wrapped)?;
		Self::try_remove_para_id(original, para_id)?;

		OriginalSystemTokenConverter::<T>::remove(&wrapped);
		SystemTokenProperties::<T>::remove(&wrapped);

		Ok(())
	}

	/// **Description:**
	///
	/// Try suspend for all `original` and `wrapped` system tokens registered on runtime.
	///
	/// **Changes:**
	fn try_suspend_all(original: SystemTokenId) -> DispatchResult {
		let wrapped_system_tokens = Self::try_get_wrapped_system_token_list(&original)?;
		for wrapped_system_token_id in wrapped_system_tokens {
			Self::try_suspend(&wrapped_system_token_id, true)?;
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
	fn try_suspend(
		wrapped: &SystemTokenId,
		is_allowed_to_suspend_original: bool,
	) -> DispatchResult {
		let original = OriginalSystemTokenConverter::<T>::get(&wrapped)
			.ok_or(Error::<T>::WrappedNotRegistered)?;

		let SystemTokenId { para_id, .. } = wrapped.clone();
		if original.para_id == para_id && !is_allowed_to_suspend_original {
			return Err(Error::<T>::BadAccess.into())
		}

		SystemTokenProperties::<T>::try_mutate_exists(&wrapped, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.status = SystemTokenStatus::Suspended;
			*p = Some(property);
			Ok(())
		})?;

		Self::try_demote(*wrapped, false)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try unsuspend for all `original` and `wrapped` system tokens registered on runtime.
	///
	/// **Changes:**
	fn try_unsuspend_all(original: SystemTokenId) -> DispatchResult {
		let wrapped_system_tokens = Self::try_get_wrapped_system_token_list(&original)?;
		for wrapped_system_token_id in wrapped_system_tokens {
			Self::try_unsuspend(&wrapped_system_token_id, true)?;
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
	fn try_unsuspend(
		wrapped: &SystemTokenId,
		is_allowed_to_unsuspend_original: bool,
	) -> DispatchResult {
		let original = OriginalSystemTokenConverter::<T>::get(&wrapped)
			.ok_or(Error::<T>::WrappedNotRegistered)?;

		let SystemTokenId { para_id, .. } = wrapped.clone();
		if original.para_id == para_id && !is_allowed_to_unsuspend_original {
			return Err(Error::<T>::BadAccess.into())
		}

		SystemTokenProperties::<T>::try_mutate_exists(&wrapped, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.status = SystemTokenStatus::Active;
			*p = Some(property);
			Ok(())
		})?;

		Self::try_promote(&wrapped, None)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try update `weight` property of `original` system token
	///
	/// **Changes:**
	///
	/// - `SystemTokenProperties`
	fn try_update_weight_property(
		system_token_id: &SystemTokenId,
		system_token_weight: SystemTokenWeight,
	) -> DispatchResult {
		SystemTokenProperties::<T>::try_mutate_exists(system_token_id, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.system_token_weight = Some(system_token_weight);
			*p = Some(property.clone());
			Self::deposit_event(Event::<T>::SetSystemTokenWeight {
				system_token_id: system_token_id.clone(),
				property,
			});
			Ok(())
		})?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try push `system_token_id` for given `para_id` to `ParaIdSystemTokens`.
	///
	/// **Errors:**
	///
	/// - `TooManySystemTokensOnPara`: Maximum number of elements has been reached for BoundedVec
	fn try_push_sys_token_for_para_id(
		para_id: SystemTokenParaId,
		system_token_id: &SystemTokenId,
	) -> DispatchResult {
		ParaIdSystemTokens::<T>::try_mutate_exists(
			para_id.clone(),
			|maybe_used_system_tokens| -> Result<(), DispatchError> {
				let mut system_tokens = maybe_used_system_tokens
					.take()
					.map_or(Default::default(), |sys_tokens| sys_tokens);
				system_tokens
					.try_push(*system_token_id)
					.map_err(|_| Error::<T>::TooManySystemTokensOnPara)?;
				*maybe_used_system_tokens = Some(system_tokens);
				Ok(())
			},
		)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try push `para_id` for given `original` system token to `SystemTokenUsedParaIds`
	///
	/// **Errors:**
	///
	/// - `TooManyUsed`: Number of para ids using `original` system tokens has reached
	///   `MaxSystemTokenUsedParaIds`
	fn try_push_para_id(para_id: SystemTokenParaId, original: &SystemTokenId) -> DispatchResult {
		SystemTokenUsedParaIds::<T>::try_mutate_exists(
			original,
			|maybe_para_id_list| -> Result<(), DispatchError> {
				let mut para_ids = maybe_para_id_list.take().map_or(Default::default(), |ids| ids);
				para_ids.try_push(para_id).map_err(|_| Error::<T>::TooManyUsed)?;
				*maybe_para_id_list = Some(para_ids);
				Ok(())
			},
		)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try remove `ParaIdSystemTokens` for any(`original` or `wrapped`) system token id
	fn try_remove_sys_token_for_para(sys_token_id: &SystemTokenId) -> DispatchResult {
		let SystemTokenId { para_id, .. } = sys_token_id.clone();
		ParaIdSystemTokens::<T>::try_mutate_exists(
			para_id,
			|maybe_system_tokens| -> Result<(), DispatchError> {
				let mut system_tokens =
					maybe_system_tokens.take().ok_or(Error::<T>::ParaIdSystemTokensNotFound)?;
				system_tokens.retain(|&x| x != *sys_token_id);
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

	/// **Description:**
	///
	/// Try remove `SystemTokenUsedParaIds` for system token id
	fn try_remove_para_id(original: SystemTokenId, para_id: SystemTokenParaId) -> DispatchResult {
		SystemTokenUsedParaIds::<T>::try_mutate_exists(
			original,
			|maybe_para_ids| -> Result<(), DispatchError> {
				let mut para_ids = maybe_para_ids.take().ok_or(Error::<T>::ParaIdsNotFound)?;
				para_ids.retain(|x| *x != para_id);
				if para_ids.is_empty() {
					*maybe_para_ids = None;
					return Ok(())
				}
				*maybe_para_ids = Some(para_ids);
				Ok(())
			},
		)?;

		Ok(())
	}
}

// XCM-related internal methods
impl<T: Config> Pallet<T> {
	/// **Description:**
	///
	/// Try change state of `wrapped` system token, which is `sufficient` and unlink the system
	/// token with local asset
	///
	/// **Params:**
	///
	/// - wrapped: Wrapped system token expected to be changed
	///
	/// - is_sufficient: State of 'sufficient' to be changed
	///
	/// **Logic:**
	///
	/// If `para_id == 0`, which means Relay Chain, call internal `Assets` pallet method.
	/// Otherwise, send DMP of `demote` to expected `para_id`
	/// destination
	fn try_demote(wrapped: SystemTokenId, is_unlink: bool) -> DispatchResult {
		let SystemTokenId { para_id, asset_id, .. } = wrapped;
		T::InfraCore::deregister_system_token(para_id, asset_id, is_unlink);

		Ok(())
	}

	/// **Description:**
	///
	/// Try sending DMP of call `promote` to specific parachain.
	/// If success, destination parachain's local asset's `sufficient` state to `is_sufficient`, and
	/// set its weight
	fn try_promote(
		system_token_id: &SystemTokenId,
		system_token_weight: Option<InfraSystemTokenWeightOf<T>>,
	) -> DispatchResult {
		let SystemTokenId { para_id, asset_id, .. } = system_token_id.clone();
		let weight = system_token_weight.ok_or(Error::<T>::WeightMissing)?;
		T::InfraCore::register_system_token(para_id, asset_id, weight);
		T::InfraCore::update_runtime_state(para_id);
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
		wrapped: &SystemTokenIdOf<T>,
		system_token_weight: InfraSystemTokenWeightOf<T>,
	) -> DispatchResult {
		let original = <OriginalSystemTokenConverter<T>>::get(wrapped)
			.ok_or(Error::<T>::WrappedNotRegistered)?;
		let original_metadata =
			OriginalSystemTokenMetadata::<T>::get(&original).ok_or(Error::<T>::MetadataNotFound)?;
		let SystemTokenId { para_id, asset_id, .. } = wrapped.clone();
		let parent_for_asset_link: u8 = if para_id == RELAY_CHAIN_PARA_ID { 0 } else { 1 };
		T::InfraCore::create_wrapped_local(
			para_id,
			asset_id,
			original_metadata.currency_type,
			original_metadata.min_balance,
			original_metadata.name.to_vec(),
			original_metadata.symbol.to_vec(),
			original_metadata.decimals,
			system_token_weight,
			parent_for_asset_link,
			original,
		);
		Ok(())
	}
}

pub mod types {

	use super::*;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	pub type InfraAssetIdOf<T> = <<T as Config>::InfraCore as UpdateInfraConfig>::AssetId;
	pub type InfraBalanceOf<T> = <<T as Config>::InfraCore as UpdateInfraConfig>::Balance;
	pub type InfraParaIdOf<T> = <<T as Config>::InfraCore as UpdateInfraConfig>::ParaId;
	pub type InfraSystemTokenWeightOf<T> = <<T as Config>::InfraCore as UpdateInfraConfig>::SystemTokenWeight;
	pub type SystemTokenIdOf<T> = <T as Config>::SystemTokenId;
	pub type BoundedStringOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;

	// impl TryFrom<MultiLocation> for SystemTokenId {
	// 	type Error = SystemTokenError;
	// 	fn try_from(value: MultiLocation) -> Result<Self, Self::Error> {
	// 		match value {
	// 			Junctions::X3(Junction::Parachain(para_id), Junction::PalletInstance(pallet_id), Junction::GeneralIndex(asset_id)) => {
	// 				Ok(SystemTokenId::new(Some(para_id), pallet_id, asset_id))
	// 			},
	// 			_ => Err(SystemTokenError::ConvertError)
	// 		}
	// 	}
	// }

	// impl Into<MultiLocation> for SystemTokenId {
	// 	fn into(self) -> MultiLocation {
	// 		match self.para_id {
	// 			Some(_) => MultiLocation { parents: 0, interior: Junctions::X3(Junction::Parachain(self.para_id), Junction::PalletInstance(self.pallet_id), Junction::GeneralIndex(self.asset_id))},
	// 			None => MultiLocation { parents: 0, interior: Junctions::X2(Junction::PalletInstance(self.pallet_id), Junction::GeneralIndex(self.asset_id))}
	// 		}
	// 	}
	// }

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum SystemTokenType<Id> {
		Original(Id),
		Wrapped { original: Id, wrapped: Id },
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

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ParaCallMetadata {
		pub(crate) para_id: InfraParaIdOf<T>,
		pub(crate) pallet_id: InfraPalletIdOf<T>,
		pub(crate) pallet_name: Vec<u8>,
		pub(crate) call_name: Vec<u8>,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ExtendedMetadata {
		/// The user friendly name of issuer in real world
		pub(crate) issuer: Option<Vec<u8>>,
		/// The description of the token
		pub(crate) description: Option<Vec<u8>>,
		/// The url of related to the token or issuer
		pub(crate) url: Option<Vec<u8>>,
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	/// Metadata of the `original` asset from enshrined runtime
	// TODO: `SystemTokenMetadata` -> `SystemTokenDetail`
	// TODO: Additional fields -> `SystemTokenWeight`, `List of Wrapped`, `Status`
	pub struct SystemTokenMetadata<BoundedString, Balance> {
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
		pub(crate) min_balance: Balanace,
		/// The unix time of this system token registered
		#[codec(compact)]
		pub(crate) registered_at: u128,
		/// The user friendly name of issuer in real world
		pub(crate) issuer: Option<BoundedString>,
		/// The description of the token
		pub(crate) description: Option<BoundedString>,
		/// The url of related to the token or issuer
		pub(crate) url: Option<BoundedString>,
	}

	impl<BoundedString: Default, Balance> SystemTokenMetadata<BoundedString, Balance> {
		pub fn currency_type(&self) -> Fiat {
			self.currency_type.clone()
		}

		pub fn set_registered_at(&mut self, at: u128) {
			self.registered_at = at;
		}

		pub fn new(
			currency_type: Fiat,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			min_balance: Balance,
		) -> Self {
			Self {
				currency_type,
				name,
				symbol,
				decimals,
				min_balance,
				issuer: None,
				description: None,
				url: None,
				..Default::default()
			}
		}

		pub fn additional(
			&mut self,
			issuer: Option<BoundedString>,
			description: Option<BoundedString>,
			url: Option<BoundedString>,
		) {
			self.issuer = issuer;
			self.description = description;
			self.url = url;
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct SystemTokenProperty {
		pub(crate) system_token_weight: Option<SystemTokenWeight>,
		pub(crate) status: SystemTokenStatus,
		pub(crate) created_at: u128,
	}
}

mod impl_traits {
	use super::*;

	impl<T: Config> SystemTokenInterface<VoteWeight, RemoteAssetMetadata<>> for Pallet<T> {

		type Id = SystemTokenIdOf<T>;
		type Balance = InfraBalanceOf<T>;

		fn is_system_token(original: &Self::Id) -> bool {
			<OriginalSystemTokenMetadata<T>>::get(original).is_some()
		}
		fn convert_to_original_system_token(wrapped: &Self::Id) -> Option<Self::Id> {
			if let Some(original) = <OriginalSystemTokenConverter<T>>::get(wrapped) {
				Self::deposit_event(Event::<T>::SystemTokenConverted {
					from: wrapped.clone(),
					to: original.clone(),
				});
				return Some(original)
			}
			None
		}
		fn adjusted_weight(original: &Self::Id, vote_weight: VoteWeight) -> VoteWeight {
			if let Some(p) = <SystemTokenProperties<T>>::get(original) {
				if let Ok(infra_system_config) = T::InfraCore::infra_system_config() {
					let system_token_weight = {
						let w: u128 =
							p.system_token_weight.map_or(infra_system_config.base_weight(), |w| w);
						let system_token_weight = F64::from_i128(w as i128);
						system_token_weight
					};
					let converted_base_weight =
						F64::from_i128(infra_system_config.base_weight() as i128);
	
					// Since the base_weight cannot be zero, this division is guaranteed to be safe.
					return vote_weight.mul(system_token_weight).div(converted_base_weight)
				}
				return vote_weight
			}
			vote_weight
		}
		fn requested_asset_metadata(
			para_id: InfraParaIdOf<T>,
			maybe_requested_asset: Option<RemoteAssetMetadata<Self::AssetId, Self::Balance>>,
		) {
			if let Some(request_asset_metadata) = maybe_requested_asset {
				let RemoteAssetMetadata {
					asset_id,
					name,
					symbol,
					currency_type,
					decimals,
					min_balance,
				} = request_asset_metadata;
				let system_token_id = SystemTokenId::new(para_id, 0, asset_id); // TODO: Remove `pallet_id`
				OriginalSystemTokenMetadata::<T>::insert(
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
			}
		}
	}

	impl SystemTokenInterface<VoteWeight> for () {

		type AssetId = ();
		type Balance = ();

		fn is_system_token(_system_token: &SystemTokenId) -> bool {
			false
		}
		fn convert_to_original_system_token(_wrapped_token: &SystemTokenId) -> Option<SystemTokenId> {
			None
		}
		fn adjusted_weight(_system_token: &SystemTokenId, _vote_weight: VoteWeight) -> VoteWeight {
			Default::default()
		}
		fn requested_asset_metadata(
			_para_id: ParaId,
			_maybe_requested_asset: Option<RemoteAssetMetadata<AssetId, Balance>>,
		) {
		}
	}
}
