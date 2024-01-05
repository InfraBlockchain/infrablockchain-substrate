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

use crate::{configuration, dmp, paras, system_token_helper};
pub use frame_support::{
	pallet_prelude::*,
	traits::{infra_support::system_token::SystemTokenInterface, UnixTime},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_asset_link::AssetIdOf;
use softfloat::F64;
use sp_runtime::{
	traits::StaticLookup,
	types::{
		AssetId as InfraAssetId, PalletId as InfraPalletId, ParaId as InfraParaId, SystemTokenId,
		SystemTokenWeight, VoteWeight, RELAY_CHAIN_PARA_ID,
	},
};
use sp_std::prelude::*;
use types::*;

use pallet_transaction_payment::BalanceOf;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ configuration::Config
		+ paras::Config
		+ dmp::Config
		+ pallet_assets::Config
		+ pallet_asset_link::Config
		+ pallet_system_token_tx_payment::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
		SetSystemTokenWeight { original: SystemTokenId, property: SystemTokenProperty },
		/// Update the fee rate of the parachain. The default value is 1_000.
		SetParaFeeRate { para_id: InfraParaId, para_fee_rate: BalanceOf<T> },
		/// Update the fee table of the parachain
		SetFeeTable { para_call_metadata: ParaCallMetadata, fee: BalanceOf<T> },
		/// Suspend a `original` system token.
		OriginalSystemTokenSuspended { original: SystemTokenId },
		/// Unsuspend the `original` system token.
		OriginalSystemTokenUnsuspended { original: SystemTokenId },
		/// Suspend a `wrapped` system token.
		WrappedSystemTokenSuspended { wrapped: SystemTokenId },
		/// Unsuspend the `wrapped` system token.
		WrappedSystemTokenUnsuspended { wrapped: SystemTokenId },
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
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

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
	/// Metadata(`SystemTokenMetadata`, `AssetMetadata`)
	pub type OriginalSystemTokenMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, SystemTokenId, IbsSystemTokenMetadata<T>, OptionQuery>;

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
		StorageMap<_, Blake2_128Concat, SystemTokenId, SystemTokenProperty, OptionQuery>;

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
		StorageMap<_, Blake2_128Concat, SystemTokenId, SystemTokenId, OptionQuery>;

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
		InfraParaId,
		BoundedVec<SystemTokenId, T::MaxSystemTokens>,
		OptionQuery,
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
		SystemTokenId,
		BoundedVec<InfraParaId, T::MaxOriginalUsedParaIds>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_assets::Config>::AssetIdParameter: From<InfraAssetId>,
		AssetIdOf<T>: From<InfraAssetId>,
	{
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
		// `try_register_original` and `try_set_sufficient_and_weight` will be called,
		// which will change `original` system token state
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			system_token_type: SystemTokenType,
			system_token_weight: SystemTokenWeight,
			wrapped_for_relay_chain: Option<SystemTokenId>,
			system_token_metadata: Option<SystemTokenMetadata<BoundedVec<u8, StringLimitOf<T>>>>,
			asset_metadata: Option<AssetMetadata<BoundedVec<u8, StringLimitOf<T>>, T::Balance>>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let (original, wrapped) = match system_token_type {
				SystemTokenType::Original(original) => {
					let system_token_metadata =
						system_token_metadata.ok_or(Error::<T>::SystemTokenMetadataNotProvided)?;
					let asset_metadata =
						asset_metadata.ok_or(Error::<T>::AssetMetadataNotProvided)?;
					Self::try_register_original(
						&original,
						system_token_weight,
						system_token_metadata,
						asset_metadata,
					)?;
					let wrapped =
						wrapped_for_relay_chain.ok_or(Error::<T>::WrappedForRelayNotProvided)?;
					ensure!(wrapped.para_id == RELAY_CHAIN_PARA_ID, Error::<T>::BadAccess);
					(original, wrapped)
				},
				SystemTokenType::Wrapped { original, wrapped } => {
					ensure!(
						!wrapped_for_relay_chain.is_some() &&
							!system_token_metadata.is_some() &&
							!asset_metadata.is_some(),
						Error::<T>::BadAccess
					);

					(original, wrapped)
				},
			};
			// Create wrapped for Relay Chain
			let system_token_weight = Self::try_register_wrapped(&original, &wrapped)?;
			Self::try_create_wrapped(wrapped, system_token_weight)?;

			Self::deposit_event(Event::<T>::OriginalSystemTokenRegistered { original });
			Self::deposit_event(Event::<T>::WrappedSystemTokenRegistered { original, wrapped });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
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
			system_token_type: SystemTokenType,
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
		#[pallet::weight(1_000)]
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
			system_token_type: SystemTokenType,
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
		#[pallet::weight(1_000)]
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
		#[pallet::weight(1_000)]
		// Description:
		// Update the weight of system token, which can affect on PoT vote
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - original: System token id expected to be updated
		// - system_token_weight: System token weight expected to be updated
		pub fn update_system_token_weight(
			origin: OriginFor<T>,
			original: SystemTokenId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;

			let wrapped_system_tokens = Self::try_get_wrapped_system_token_list(&original)?;
			for wrapped_system_token_id in wrapped_system_tokens.iter() {
				Self::try_update_weight_of_wrapped(wrapped_system_token_id, system_token_weight)?;
			}
			Self::try_update_weight_property_of_original(original, system_token_weight)?;

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(1_000)]
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
			para_id: InfraParaId,
			pallet_id: InfraPalletId,
			para_fee_rate: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let encoded_call: Vec<u8> =
				pallet_system_token_tx_payment::Call::<T>::set_para_fee_rate { para_fee_rate }
					.encode();
			system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;
			Self::deposit_event(Event::<T>::SetParaFeeRate { para_id, para_fee_rate });

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(1_000)]
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
		pub fn set_fee_table(
			origin: OriginFor<T>,
			para_call_metadata: ParaCallMetadata,
			fee: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let ParaCallMetadata { para_id, pallet_id, pallet_name, call_name } =
				para_call_metadata.clone();
			let encoded_call: Vec<u8> = pallet_system_token_tx_payment::Call::<T>::set_fee_table {
				pallet_name,
				call_name,
				fee,
			}
			.encode();
			system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;

			Self::deposit_event(Event::<T>::SetFeeTable { para_call_metadata, fee });

			Ok(())
		}
	}
}

// System token related interal methods
impl<T: Config> Pallet<T>
where
	<T as pallet_assets::Config>::AssetIdParameter: From<InfraAssetId>,
	AssetIdOf<T>: From<InfraAssetId>,
{
	fn unix_time() -> u128 {
		T::UnixTime::now().as_millis()
	}

	fn system_token_metadata(
		system_token_metdata: SystemTokenMetadata<BoundedVec<u8, StringLimitOf<T>>>,
	) -> Result<SystemTokenMetadata<BoundedVec<u8, StringLimitOf<T>>>, DispatchError> {
		let SystemTokenMetadata { issuer, description, url } = system_token_metdata;
		let issuer = issuer.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let description = description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let url = url.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		Ok(SystemTokenMetadata { issuer, description, url })
	}

	fn asset_metadata(
		asset_metadata: AssetMetadata<BoundedVec<u8, StringLimitOf<T>>, T::Balance>,
	) -> Result<AssetMetadata<BoundedVec<u8, StringLimitOf<T>>, T::Balance>, DispatchError> {
		let AssetMetadata { name, symbol, decimals, min_balance } = asset_metadata;

		let name = name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let symbol = symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		Ok(AssetMetadata { name, symbol, decimals, min_balance })
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

	/// **Description:**
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
		original: &SystemTokenId,
		system_token_weight: SystemTokenWeight,
		system_token_metadata: SystemTokenMetadata<BoundedVec<u8, StringLimitOf<T>>>,
		asset_metadata: AssetMetadata<BoundedVec<u8, StringLimitOf<T>>, T::Balance>,
	) -> DispatchResult {
		ensure!(
			!OriginalSystemTokenMetadata::<T>::contains_key(&original),
			Error::<T>::OriginalAlreadyRegistered
		);
		let system_token_metadata = Self::system_token_metadata(system_token_metadata)?;
		let asset_metadata = Self::asset_metadata(asset_metadata)?;
		let SystemTokenId { para_id, .. } = original.clone();
		Self::try_push_sys_token_for_para_id(para_id, &original)?;
		Self::try_push_para_id(para_id, &original)?;
		Self::try_set_sufficient_and_weight(&original, true, Some(system_token_weight))?;
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

		OriginalSystemTokenMetadata::<T>::insert(
			&original,
			(&system_token_metadata, &asset_metadata),
		);

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
	) -> Result<SystemTokenWeight, DispatchError> {
		ensure!(
			OriginalSystemTokenMetadata::<T>::contains_key(original),
			Error::<T>::OriginalNotRegistered
		);
		ensure!(
			!OriginalSystemTokenConverter::<T>::contains_key(wrapped),
			Error::<T>::WrappedAlreadyRegistered
		);

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
		Ok(weight)
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
		if original.para_id == para_id {
			OriginalSystemTokenMetadata::<T>::remove(original);
			Self::try_set_sufficient_and_weight(&original, false, None)?;
		} else {
			Self::try_set_sufficient_and_unlink(wrapped.clone(), false)?;
		}

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

		let property =
			SystemTokenProperties::<T>::get(original).ok_or(Error::<T>::PropertyNotFound)?;

		ensure!(property.status != SystemTokenStatus::Suspended, Error::<T>::AlreadySuspended);

		SystemTokenProperties::<T>::try_mutate_exists(&wrapped, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.status = SystemTokenStatus::Suspended;
			*p = Some(property);
			Ok(())
		})?;

		Self::try_set_sufficient_and_weight(&wrapped, false, property.system_token_weight)?;

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
		let property =
			SystemTokenProperties::<T>::get(original).ok_or(Error::<T>::PropertyNotFound)?;

		ensure!(property.status == SystemTokenStatus::Suspended, Error::<T>::NotSuspended);

		SystemTokenProperties::<T>::try_mutate_exists(&wrapped, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.status = SystemTokenStatus::Active;
			*p = Some(property);
			Ok(())
		})?;

		Self::try_set_sufficient_and_weight(&wrapped, true, property.system_token_weight)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try update `weight` property of `original` system token
	///
	/// **Changes:**
	///
	/// - `SystemTokenProperties`
	fn try_update_weight_property_of_original(
		original: SystemTokenId,
		system_token_weight: SystemTokenWeight,
	) -> DispatchResult {
		SystemTokenProperties::<T>::try_mutate_exists(&original, |p| -> DispatchResult {
			let mut property = p.take().ok_or(Error::<T>::PropertyNotFound)?;
			property.system_token_weight = Some(system_token_weight);
			*p = Some(property.clone());
			Self::deposit_event(Event::<T>::SetSystemTokenWeight { original, property });
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
		para_id: InfraParaId,
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
	fn try_push_para_id(para_id: InfraParaId, original: &SystemTokenId) -> DispatchResult {
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
	fn try_remove_para_id(original: SystemTokenId, para_id: InfraParaId) -> DispatchResult {
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
impl<T: Config> Pallet<T>
where
	<T as pallet_assets::Config>::AssetIdParameter: From<InfraAssetId>,
	AssetIdOf<T>: From<InfraAssetId>,
{
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
	/// Otherwise, send DMP of `set_sufficient_with_unlink_system_token` to expected `para_id`
	/// destination
	fn try_set_sufficient_and_unlink(
		wrapped: SystemTokenId,
		is_sufficient: bool,
	) -> DispatchResult {
		let SystemTokenId { para_id, pallet_id, asset_id } = wrapped;
		if para_id == 0u32 {
			// Relay Chain
			pallet_assets::pallet::Pallet::<T>::do_set_sufficient_and_unlink(
				&asset_id.into(),
				false,
			)?;
		} else {
			// Parachain
			let encoded_call = pallet_assets::Call::<T>::set_sufficient_with_unlink_system_token {
				id: asset_id.into(),
				is_sufficient,
			}
			.encode();
			system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;
		}

		Ok(())
	}

	/// **Description:**
	///
	/// Try sending DMP of call `set_sufficient_and_system_token_weight` to specific parachain.
	/// If success, destination parachain's local asset's `sufficient` state to `is_sufficient`, and
	/// set its weight
	fn try_set_sufficient_and_weight(
		system_token_id: &SystemTokenId,
		is_sufficient: bool,
		system_token_weight: Option<SystemTokenWeight>,
	) -> DispatchResult {
		let SystemTokenId { para_id, pallet_id, asset_id } = system_token_id.clone();
		let encoded_call = pallet_assets::Call::<T>::set_sufficient_and_system_token_weight {
			id: asset_id.into(),
			is_sufficient,
			system_token_weight,
		}
		.encode();
		system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;

		Ok(())
	}

	/// **Description:**
	///
	/// Try update weight of `wrapped` system token.
	///
	/// **Params:**
	///
	/// - wrapped: `wrapped` system token expected to be updated
	///
	/// - system_token_weight: Weight expected to be updated
	///
	/// **Logic:**
	///
	/// If `para_id == 0`, call internal `Assets` pallet method.
	/// Otherwise, send DMP of `update_system_token_weight` to expected `para_id` destination
	fn try_update_weight_of_wrapped(
		wrapped: &SystemTokenId,
		system_token_weight: SystemTokenWeight,
	) -> DispatchResult {
		let SystemTokenId { para_id, pallet_id, asset_id } = wrapped.clone();

		if para_id == 0u32 {
			// Relay Chain
			pallet_assets::pallet::Pallet::<T>::do_update_system_token_weight(
				asset_id.into(),
				system_token_weight,
			)?
		} else {
			// Parachain
			let encoded_call = pallet_assets::Call::<T>::update_system_token_weight {
				id: asset_id.into(),
				system_token_weight,
			}
			.encode();
			system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;
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
		wrapped: SystemTokenId,
		system_token_weight: SystemTokenWeight,
	) -> DispatchResult {
		let original = <OriginalSystemTokenConverter<T>>::get(&wrapped)
			.ok_or(Error::<T>::WrappedNotRegistered)?;
		let (_, asset_metadata) =
			OriginalSystemTokenMetadata::<T>::get(&original).ok_or(Error::<T>::MetadataNotFound)?;
		let SystemTokenId { para_id, pallet_id, asset_id } = wrapped;
		let root = system_token_helper::root_account::<T>();
		if para_id == RELAY_CHAIN_PARA_ID {
			// Relay Chain
			pallet_assets::pallet::Pallet::<T>::do_create_asset_with_metadata(
				asset_id.into(),
				T::Lookup::unlookup(root),
				true,
				asset_metadata.min_balance,
				asset_metadata.name.to_vec(),
				asset_metadata.symbol.to_vec(),
				asset_metadata.decimals,
				false,
				original,
				0,
				system_token_weight,
			)?;
			pallet_system_token_tx_payment::pallet::Pallet::<T>::do_set_runtime_state()?;
		} else {
			// Parachain
			let encoded_call: Vec<u8> = pallet_assets::Call::<T>::force_create_with_metadata {
				id: asset_id.into(),
				owner: T::Lookup::unlookup(root),
				min_balance: asset_metadata.min_balance,
				is_sufficient: true,
				name: asset_metadata.name.to_vec(),
				symbol: asset_metadata.symbol.to_vec(),
				decimals: asset_metadata.decimals,
				is_frozen: false,
				system_token_id: original,
				asset_link_parents: 1,
				system_token_weight,
			}
			.encode();

			system_token_helper::try_queue_dmp::<T>(para_id, pallet_id, encoded_call)?;
		}

		Ok(())
	}
}

impl<T: Config> SystemTokenInterface for Pallet<T> {
	fn is_system_token(original: &SystemTokenId) -> bool {
		<OriginalSystemTokenMetadata<T>>::get(original).is_some()
	}
	fn convert_to_original_system_token(wrapped: &SystemTokenId) -> Option<SystemTokenId> {
		if let Some(original) = <OriginalSystemTokenConverter<T>>::get(wrapped) {
			Self::deposit_event(Event::<T>::SystemTokenConverted {
				from: wrapped.clone(),
				to: original.clone(),
			});
			return Some(original)
		}
		None
	}
	fn adjusted_weight(original: &SystemTokenId, vote_weight: VoteWeight) -> VoteWeight {
		if let Some(p) = <SystemTokenProperties<T>>::get(original) {
			let system_token_weight = {
				let w: u128 = p.system_token_weight.map_or(BASE_WEIGHT, |w| w);
				let system_token_weight = F64::from_i128(w as i128);
				system_token_weight
			};
			let base_weight = F64::from_i128(BASE_WEIGHT as i128);
			// Since the base_weight cannot be zero, this division is guaranteed to be safe.
			return vote_weight.mul(system_token_weight).div(base_weight)
		}
		vote_weight
	}
}

pub mod types {

	use super::*;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	pub type StringLimitOf<T> = <T as Config>::StringLimit;
	pub type IbsSystemTokenMetadata<T> = (
		SystemTokenMetadata<BoundedVec<u8, StringLimitOf<T>>>,
		AssetMetadata<BoundedVec<u8, StringLimitOf<T>>, <T as pallet_assets::Config>::Balance>,
	);

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum SystemTokenType {
		Original(SystemTokenId),
		Wrapped { original: SystemTokenId, wrapped: SystemTokenId },
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

	pub const BASE_WEIGHT: u128 = 1_000_000;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ParaCallMetadata {
		pub(crate) para_id: InfraParaId,
		pub(crate) pallet_id: InfraPalletId,
		pub(crate) pallet_name: Vec<u8>,
		pub(crate) call_name: Vec<u8>,
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]

	/// Metadata of the `original` system token for Relay Chain
	pub struct SystemTokenMetadata<BoundedString> {
		/// The user friendly name of issuer in real world
		pub(crate) issuer: BoundedString,
		/// The description of the token
		pub(crate) description: BoundedString,
		/// The url of related to the token or issuer
		pub(crate) url: BoundedString,
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	/// Metadata of the `original` local asset based on parachain
	pub struct AssetMetadata<BoundedString, Balance> {
		/// The user friendly name of this system token.
		pub(crate) name: BoundedString,
		/// The exchange symbol for this system token.
		pub(crate) symbol: BoundedString,
		/// The number of decimals this asset uses to represent one unit.
		pub(crate) decimals: u8,
		/// The minimum balance of this new asset that any single account must
		/// have. If an account's balance is reduced below this, then it collapses to zero.
		pub(crate) min_balance: Balance,
	}

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct SystemTokenProperty {
		// The weight of this system token. Only 'original` system token would have its weight
		pub(crate) system_token_weight: Option<SystemTokenWeight>,
		// The status of this systen token
		pub(crate) status: SystemTokenStatus,
		// The epoch time of this system token registered
		pub(crate) created_at: u128,
	}
}
