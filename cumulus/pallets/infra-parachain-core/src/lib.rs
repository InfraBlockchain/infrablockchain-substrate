#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_pallet_xcm::{ensure_relay, Origin};
use cumulus_primitives_core::UpdateRCConfig;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{types::{fee::*, infra_core::*, token::*, vote::*}, Saturating};
use sp_std::vec::Vec;
use scale_info::TypeInfo;

pub use pallet::*;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum RequestStatus<BlockNumber> {
	Available,
	Requested { at: BlockNumber, is_relay: bool }
}

impl<BlockNumber> RequestStatus<BlockNumber> 
where
	BlockNumber: Saturating + Ord + PartialOrd
{

	pub fn default_status(at: BlockNumber) -> Self {
		Self::Requested { at, is_relay: false}
	}

	fn is_relayed(&mut self) -> bool {
		match self {
			Self::Requested { is_relay, .. } => {
				let temp = *is_relay;
				*is_relay = true;
				temp
			}
			_ => false,
		}
	}

	fn _is_available(&self) -> bool {
		match self {
			Self::Available => true,
			_ => false,
		}
	}

	fn is_expired(self, current: BlockNumber, active_period: BlockNumber) -> bool {
		match self {
			Self::Requested { at, .. } => {
				let exp = at.saturating_add(active_period);
				current >= exp
			}
			_ => false,
		}
	}
}

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
		type LocalAssetManager: LocalAssetManager<AccountId = Self::AccountId>;
		/// Type that links local asset with System Token
		type AssetLink: AssetLinkInterface<SystemTokenAssetId>;
		/// Type that interacts with Parachain System
		type ParachainSystem: CollectVote + AssetMetadataProvider;
		/// Active request period for registering System Token
		#[pallet::constant]
		type ActiveRequestPeriod: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type ParaCoreAdmin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	pub type RCSystemConfig<T: Config> = StorageValue<_, InfraSystemConfig>;

	#[pallet::storage]
	pub type ParaFeeRate<T: Config> = StorageValue<_, SystemTokenWeight>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> = StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance>;

	#[pallet::storage]
	pub type CurrentRequest<T: Config> = StorageValue<_, (RemoteAssetMetadata, RequestStatus<BlockNumberFor<T>>)>;

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
		ParaCoreAdminUpdated { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Mode of Runtime cannot be changed(e.g SystemTokenMissing)
		NotAllowedToChangeState,
		/// System Token is not registered
		SystemTokenMissing,
		/// System Token has not been requested
		SystemTokenNotRequested,
		/// Runtime has not been initiated(e.g BaseConfigMissing)
		NotInitiated,
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
		/// Error occured while getting metadata
		ErrorOnGetMetadata,
		/// Error occured while requesting register
		ErrorOnRequestRegister,
		/// Currently request queue for System Token registration is fully occupied
		TooManyRequests,
		/// System Token has already been requested
		AlreadyRequested,
		/// Register is not valid
		InvalidRegister
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> frame_support::weights::Weight {
			if let Some((remote_asset_metadata, mut status)) = CurrentRequest::<T>::get() {
				if !status.is_relayed() {
					T::ParachainSystem::requested(remote_asset_metadata.clone());
					CurrentRequest::<T>::put((remote_asset_metadata, status));
				}
				T::DbWeight::get().reads_writes(1, 1)
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Fee table for Runtime will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(1)]
		pub fn update_fee_table(
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
		pub fn update_para_fee_rate(
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
		pub fn update_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			if RuntimeState::<T>::get() == Mode::Normal {
				return Ok(())
			}
			ensure!(RCSystemConfig::<T>::get().is_some(), Error::<T>::NotAllowedToChangeState);
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
		#[pallet::call_index(4)]
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
		#[pallet::call_index(5)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			Self::check_valid_register()?;
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
		#[pallet::call_index(6)]
		pub fn create_wrapped_local(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			currency_type: Fiat,
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

		#[pallet::call_index(7)]
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
		/// It can call extrinsic which is not allowed to call by other origin(e.g
		/// `request_register_system_token`)
		#[pallet::call_index(8)]
		pub fn set_para_core_admin(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			ParaCoreAdmin::<T>::put(&who);
			Self::deposit_event(Event::<T>::ParaCoreAdminUpdated { who });
			Ok(())
		}

		/// Request to register System Token
		///
		/// If succeed, request will be queued in `RequestQueue`
		#[pallet::call_index(9)]
		pub fn request_register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
		) -> DispatchResult {
			if let Some(acc) = ensure_signed_or_root(origin)? {
				ensure!(ParaCoreAdmin::<T>::get() == Some(acc), Error::<T>::NoPermission);
			}
			let remote_asset_metadata = T::LocalAssetManager::get_metadata(asset_id)
				.map_err(|_| Error::<T>::ErrorOnGetMetadata)?;
			T::LocalAssetManager::request_register(asset_id)
				.map_err(|_| Error::<T>::ErrorOnRequestRegister)?;
			Self::do_request(remote_asset_metadata)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_request(asset_metadata: RemoteAssetMetadata) -> Result<(), DispatchError> {
		let current = <frame_system::Pallet<T>>::block_number();
		if let Some((_, request_status)) = CurrentRequest::<T>::get() {
			let active_period = T::ActiveRequestPeriod::get();
			if request_status.is_expired(current, active_period) {
				return Err(Error::<T>::AlreadyRequested.into())
			}
		} 
		CurrentRequest::<T>::put((asset_metadata, RequestStatus::default_status(current)));
		Ok(())
	}

	fn check_valid_register() -> Result<(), DispatchError> {
		let is_valid = if let Some((_, status)) = CurrentRequest::<T>::get() {
			if !status.is_expired(
				<frame_system::Pallet<T>>::block_number(),
				T::ActiveRequestPeriod::get(),
			) {
				CurrentRequest::<T>::kill();
				true
			} else {
				false
			}
		} else {
			false
		};

		ensure!(is_valid, Error::<T>::InvalidRegister);
		Ok(())
	}
}

impl<T: Config> RuntimeConfigProvider for Pallet<T> {
	type Error = DispatchError;

	fn infra_system_config() -> Result<InfraSystemConfig, Self::Error> {
		Ok(RCSystemConfig::<T>::get().ok_or(Error::<T>::NotInitiated)?)
	}

	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error> {
		let base_weight = RCSystemConfig::<T>::get().ok_or(Error::<T>::NotInitiated)?.base_weight();
		Ok(ParaFeeRate::<T>::try_mutate_exists(
			|maybe_para_fee_rate| -> Result<SystemTokenWeight, DispatchError> {
				let pfr = maybe_para_fee_rate.take().map_or(base_weight, |pfr| pfr);
				*maybe_para_fee_rate = Some(pfr);
				Ok(pfr)
			},
		)?)
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
		T::ParachainSystem::collect_vote(who, system_token_id, vote_weight);
	}
}

impl<T: Config> UpdateRCConfig for Pallet<T> {
	fn update_system_config(infra_system_config: InfraSystemConfig) {
		RCSystemConfig::<T>::put(infra_system_config);
	}

	fn update_system_token_weight(_fiat: Fiat, _weight: SystemTokenWeight) {}
}
