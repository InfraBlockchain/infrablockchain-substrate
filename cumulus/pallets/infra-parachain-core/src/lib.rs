#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_pallet_xcm::{ensure_relay, Origin};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::types::{fee::*, infra_core::*, token::*, vote::*};
use sp_std::vec::Vec;

pub use pallet::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SystemTokenStatus {
	/// Potential System Token is requsted
	Requested,
	/// System Token is registered by RC governance
	Registered,
	/// System Token is suspended by some reasons(e.g malicious behavior detected)
	Suspend,
	/// System Token is deregistered by some reasons
	Deregistered,
}

pub type Requests<T> = BoundedVec<RemoteSystemTokenMetadata<<T as frame_system::Config>::AccountId>, <T as Config>::MaxRequests>;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Runtime Origin for the System Token pallet.
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<Origin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type that interacts with local asset
		type LocalAssetManager: LocalAssetManager<Self::AccountId>;
		/// Type that links local asset with System Token
		type AssetLink: AssetLinkInterface<SystemTokenAssetId>;
		/// Handler for TaaV
		type CollectVote: CollectVote;
		/// Maximum number of requests
		#[pallet::constant]
		type MaxRequests: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type ParaCoreOrigin<T: Config> = StorageValue<_, T::AccountId>;

	/// Base system token configuration set on Relay-chain Runtime
	#[pallet::storage]
	pub type BaseConfiguration<T: Config> = StorageValue<_, BaseSystemTokenDetail>;

	#[pallet::storage]
	pub type ParaFeeRate<T: Config> = StorageValue<_, SystemTokenWeight>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance>;

	#[pallet::storage]
	pub type RequestQueue<T: Config> = StorageValue<_, Requests<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: SystemTokenBalance },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: SystemTokenAssetId },
		/// Bootstrap has been ended by Relay-chain governance.
		BootstrapEnded,
		/// Origin of this pallet has been set by Relay-chain governance.
		SetParaCoreOrigin { who: T::AccountId }
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Mode of Runtime cannot be changed(e.g SystemTokenMissing)
		NotAllowedToChangeState,
		/// System Token is not registered
		SystemTokenMissing,
		/// Base configuration set on Relay-chain has not been set
		BaseConfigMissing,
		/// Error occured while updating weight of System Token
		ErrorUpdateWeight,
		/// Error occured while registering System Token
		ErrorRegisterSystemToken,
		/// Error occured while deregistering System Token
		ErrorDeregisterSystemToken,
		/// Error occured while creating wrapped local asset
		ErrorCreateWrappedLocalAsset,
		/// Error occured while linking asset
		ErrorLinkAsset,
		/// Error occured while unlinking asset
		ErrorUnlinkAsset,
		/// No permission to call this function
		NoPermission,
		/// Local asset does not exist
		LocalAssetNotExist,
		/// Error occured while getting metadata
		ErrorOnGetMetadata
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Base system token weight configuration will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(0)]
		pub fn set_base_config(
			origin: OriginFor<T>,
			base_system_token_detail: BaseSystemTokenDetail,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			BaseConfiguration::<T>::put(base_system_token_detail);
			Ok(())
		}

		/// Fee table for Runtime will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(1)]
		pub fn set_fee_table(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			call_name: Vec<u8>,
			fee: SystemTokenBalance,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			let extrinsic_metadata = ExtrinsicMetadata::new(pallet_name, call_name);
			FeeTable::<T>::insert(&extrinsic_metadata, fee);
			Self::deposit_event(Event::<T>::FeeTableUpdated { extrinsic_metadata, fee });
			Ok(())
		}

		/// Fee rate for Runtime will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(2)]
		pub fn set_para_fee_rate(
			origin: OriginFor<T>,
			fee_rate: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			ParaFeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set runtime state configuration for this parachain by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			if RuntimeState::<T>::get() == Mode::Normal {
				return Ok(())
			}
			ensure!(BaseConfiguration::<T>::get().is_some(), Error::<T>::NotAllowedToChangeState);
			// TODO-1: Check whether it is allowed to change `Normal` state
			// TODO-2: Check whether a parachain has enough system token to pay
			RuntimeState::<T>::put(Mode::Normal);
			Self::deposit_event(Event::<T>::BootstrapEnded);

			Ok(())
		}

		/// System Token weight configuration is set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(5)]
		pub fn update_system_token_weight(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::LocalAssetManager::update_system_token_weight(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorUpdateWeight)?;
			Ok(())
		}

		/// Register System Token for Cumulus-based parachain Runtime.
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(6)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::LocalAssetManager::promote(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Ok(())
		}

		/// Discription
		/// Asset which referes to `wrapped` System Token will be created by Relay-chain governance
		///
		/// Parameters
		/// - `asset_id`: AssetId of `wrapped` System Token
		/// - `system_token_id`: SystemTokenId of `original` System Token
		/// - `system_token_weight`: Weight of `wrapped` System Token. Need for `AssetLink`
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(7)]
		pub fn create_wrapped_local(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			currency_type: Option<Fiat>,
			min_balance: SystemTokenBalance,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			system_token_weight: SystemTokenWeight,
			asset_link_parent: u8,
			original: SystemTokenId,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::LocalAssetManager::create_wrapped_local(
				asset_id,
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight,
			)
			.map_err(|_| Error::<T>::ErrorCreateWrappedLocalAsset)?;
			T::AssetLink::link(&asset_id, asset_link_parent, original)
				.map_err(|_| Error::<T>::ErrorLinkAsset)?;
			Ok(())
		}

		#[pallet::call_index(8)]
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			is_unlink: bool,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::LocalAssetManager::demote(asset_id)
				.map_err(|_| Error::<T>::ErrorDeregisterSystemToken)?;
			if is_unlink {
				T::AssetLink::unlink(&asset_id).map_err(|_| Error::<T>::ErrorUnlinkAsset)?;
			}
			Ok(())
		}

		/// Priviliged origin governed by Relay-chain
		/// 
		/// It can call extrinsic which is not allowed to call by other origin(e.g `request_register_system_token`)
		#[pallet::call_index(9)]
		pub fn set_para_core_origin(
			origin: OriginFor<T>,
			who: T::AccountId
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			ParaCoreOrigin::<T>::put(&who);
			Self::deposit_event(Event::<T>::SetParaCoreOrigin { who });
			Ok(())
		}

		/// Request to register System Token
		/// 
		/// If succeed, request will be queued in `RequestQueue`
		#[pallet::call_index(10)]
		pub fn request_register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenId,
		) -> DispatchResult {
			if let Some(acc) = ensure_signed_or_root(origin)? {
				ensure!(ParaCoreOrigin::<T>::get() == Some(acc), Error::<T>::NoPermission);
			}
			ensure!(T::LocalAssetManager::asset_exists(asset_id.clone().into()), Error::<T>::LocalAssetNotExist);
			let remote_asset_metadata = LocalAssetManager::<T>::get_metadata(asset_id)
				.map_err(|_| Error::<T>::ErrorOnGetMetadata)?;
			Ok(())
		}
	}
}


impl<T: Config> RuntimeConfigProvider for Pallet<T> {
	type Error = DispatchError;

	fn base_system_token_configuration() -> Result<BaseSystemTokenDetail, Self::Error> {
		Ok(BaseConfiguration::<T>::get().ok_or(Error::<T>::BaseConfigMissing)?)
	}

	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error> {
		let base_system_token_detail = BaseConfiguration::<T>::get().ok_or(Error::<T>::BaseConfigMissing)?;
		Ok(
			ParaFeeRate::<T>::try_mutate_exists(|maybe_para_fee_rate| -> Result<SystemTokenWeight, DispatchError> {
				let pfr = maybe_para_fee_rate.take().map_or(base_system_token_detail.weight, |pfr| pfr);
				*maybe_para_fee_rate = Some(pfr);
				Ok(pfr)
			})?
		)
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance> {
		FeeTable::<T>::get(&ext)
	}

	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

impl<T: Config> VotingHandler for Pallet<T> {
	fn update_pot_vote(
		who: VoteAccountId,
		system_token_id: SystemTokenId,
		vote_weight: VoteWeight,
	) {
		T::CollectVote::collect_vote(who, system_token_id, vote_weight);
	}
}
