#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use cumulus_pallet_xcm::{ensure_relay, Origin};
use cumulus_primitives_core::UpdateRCConfig;
use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect, InspectSystemToken},
	},
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
	types::{fee::*, infra_core::*, token::*},
	Saturating,
};
use sp_std::vec::Vec;

pub use pallet::*;

pub type SystemTokenAssetIdOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type SystemTokenBalanceOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type SystemTokenWeightOf<T> = <<T as Config>::Fungibles as InspectSystemToken<
	<T as frame_system::Config>::AccountId,
>>::SystemTokenWeight;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct RequestStatus<BlockNumber> {
	pub exp: BlockNumber,
	pub is_relay: bool,
}

impl<BlockNumber> RequestStatus<BlockNumber>
where
	BlockNumber: Saturating + Ord + PartialOrd,
{
	pub fn default_status(exp: BlockNumber) -> Self {
		Self { exp, is_relay: false }
	}

	fn is_relayed(&mut self) -> bool {
		let temp = self.is_relay;
		if !self.is_relay {
			self.is_relay = true;
		}
		temp
	}

	fn is_expired(self, current: BlockNumber) -> bool {
		current >= self.exp
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {
		/// Runtime Origin for the System Token pallet.
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<Origin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type that interacts with local asset
		type Fungibles: InspectSystemToken<Self::AccountId>;
		/// Active request period for registering System Token
		#[pallet::constant]
		type ActiveRequestPeriod: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type ParaCoreAdmin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	pub type RCSystemConfig<T: Config> = StorageValue<_, SystemConfig>;

	#[pallet::storage]
	pub type ParaFeeRate<T: Config> = StorageValue<_, SystemTokenBalanceOf<T>>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalanceOf<T>>;

	#[pallet::storage]
	pub type CurrentRequest<T: Config> = StorageMap<
		_,
		Twox64Concat,
		SystemTokenAssetIdOf<T>,
		RequestStatus<BlockNumberFor<T>>
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// System Token has been regierested by Relay-chain governance
		Registered { asset_id: SystemTokenAssetIdOf<T> },
		/// System Token has been deregistered by Relay-chain governance
		Deregistered { asset_id: SystemTokenAssetIdOf<T> },
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: SystemTokenBalanceOf<T> },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: SystemTokenAssetIdOf<T> },
		/// Bootstrap has been ended by Relay-chain governance.
		BootstrapEnded,
		/// Origin of this pallet has been set by Relay-chain governance.
		ParaCoreAdminUpdated { who: T::AccountId },
		/// System Token registration has been requested
		RegisterRequested { asset_id: SystemTokenAssetIdOf<T>, exp: BlockNumberFor<T> },
		/// Wrapped local asset has been created
		WrappedCreated { asset_id: SystemTokenAssetIdOf<T> },
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
		/// Register is not valid(e.g Outdated registration)
		InvalidRegister,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> frame_support::weights::Weight {
			// TODO
			// if let Some(mut status) = CurrentRequest::<T>::get() {
			// 	if !status.is_relayed() {
			// 		T::ParachainSystem::requested(remote_asset_metadata.clone());
			// 		CurrentRequest::<T>::put((remote_asset_metadata, status));
			// 	}
			// 	T::DbWeight::get().reads_writes(1, 1)
			// } else {
				
			// }
			T::DbWeight::get().reads(1)
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
			fee: SystemTokenBalanceOf<T>,
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
			fee_rate: SystemTokenBalanceOf<T>,
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

		/// Register System Token for Cumulus-based parachain Runtime.
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(4)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
			system_token_weight: SystemTokenWeightOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			Self::check_valid_register(&asset_id)?;
			// T::LocalAssetManager::promote(asset_id, system_token_weight)
			// 	.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Self::deposit_event(Event::<T>::Registered { asset_id });
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
		#[pallet::call_index(5)]
		pub fn create_wrapped_local(
			origin: OriginFor<T>,
			original: SystemTokenAssetIdOf<T>,
			currency_type: Fiat,
			min_balance: SystemTokenBalanceOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			system_token_weight: SystemTokenWeightOf<T>,
			asset_link_parent: u8,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			// T::Fungibles::create_wrapped_local(
			// 	asset_id,
			// 	currency_type,
			// 	min_balance,
			// 	name,
			// 	symbol,
			// 	decimals,
			// 	system_token_weight,
			// )
			// .map_err(|_| Error::<T>::ErrorCreateWrappedLocalAsset)?;
			Self::deposit_event(Event::<T>::WrappedCreated { asset_id: original });
			Ok(())
		}

		#[pallet::call_index(6)]
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
			is_unlink: bool,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			// T::LocalAssetManager::demote(asset_id)
			// 	.map_err(|_| Error::<T>::ErrorDeregisterSystemToken)?;
			// if is_unlink {
			// 	T::AssetLink::unlink(&asset_id).map_err(|_| Error::<T>::ErrorUnlinkAsset)?;
			// }
			Self::deposit_event(Event::<T>::Deregistered { asset_id });
			Ok(())
		}

		/// Priviliged origin governed by Relay-chain
		///
		/// It can call extrinsic which is not allowed to call by other origin(e.g
		/// `request_register_system_token`)
		#[pallet::call_index(7)]
		pub fn set_para_core_admin(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			ParaCoreAdmin::<T>::put(&who);
			Self::deposit_event(Event::<T>::ParaCoreAdminUpdated { who });
			Ok(())
		}

		/// Request to register System Token
		///
		/// If succeed, request will be queued in `RequestQueue`
		#[pallet::call_index(8)]
		pub fn request_register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
		) -> DispatchResult {
			if let Some(acc) = ensure_signed_or_root(origin)? {
				ensure!(ParaCoreAdmin::<T>::get() == Some(acc), Error::<T>::NoPermission);
			}
			// let remote_asset_metadata = T::LocalAssetManager::get_metadata(asset_id)
			// 	.map_err(|_| Error::<T>::ErrorOnGetMetadata)?;
			// T::LocalAssetManager::request_register(asset_id)
			// 	.map_err(|_| Error::<T>::ErrorOnRequestRegister)?;
			// let exp = Self::do_request(asset_id, remote_asset_metadata)?;
			// Self::deposit_event(Event::<T>::RegisterRequested { asset_id, exp });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_request(
		id: SystemTokenAssetIdOf<T>,
		asset_metadata: RemoteAssetMetadata<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>,
	) -> Result<BlockNumberFor<T>, DispatchError> {
		let current = <frame_system::Pallet<T>>::block_number();
		if let Some(request_status) = CurrentRequest::<T>::get(&id) {
			if !request_status.is_expired(current) {
				return Err(Error::<T>::AlreadyRequested.into())
			}
		}
		let exp = current.saturating_add(T::ActiveRequestPeriod::get());
		CurrentRequest::<T>::insert(id, RequestStatus::default_status(exp));
		Ok(exp)
	}

	fn check_valid_register(asset: &SystemTokenAssetIdOf<T>) -> Result<(), DispatchError> {
		let is_valid = if let Some(status) = CurrentRequest::<T>::get(asset) {
			if !status.is_expired(<frame_system::Pallet<T>>::block_number()) {
				CurrentRequest::<T>::remove(asset);
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

impl<T: Config> RuntimeConfigProvider<SystemTokenBalanceOf<T>>
	for Pallet<T>
where
	SystemTokenBalanceOf<T>: From<u128>
{
	type Error = DispatchError;

	fn system_config() -> Result<SystemConfig, Self::Error> {
		Ok(RCSystemConfig::<T>::get().ok_or(Error::<T>::NotInitiated)?)
	}

	fn para_fee_rate() -> Result<SystemTokenBalanceOf<T>, Self::Error> {
		let base_weight = RCSystemConfig::<T>::get().ok_or(Error::<T>::NotInitiated)?.base_system_token_detail.base_weight;
		Ok(ParaFeeRate::<T>::try_mutate_exists(
			|maybe_para_fee_rate| -> Result<SystemTokenBalanceOf<T>, DispatchError> {
				let pfr = maybe_para_fee_rate.take().map_or(base_weight.into(), |pfr| pfr);
				*maybe_para_fee_rate = Some(pfr);
				Ok(pfr)
			},
		)?)
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalanceOf<T>> {
		FeeTable::<T>::get(&ext)
	}

	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

impl<T: Config> UpdateRCConfig<SystemTokenAssetIdOf<T>, SystemTokenWeightOf<T>> for Pallet<T> {
	fn update_system_config(system_config: SystemConfig) {
		RCSystemConfig::<T>::put(system_config);
	}

	fn update_system_token_weight_for(
		assets: Vec<(SystemTokenAssetIdOf<T>, SystemTokenWeightOf<T>)>,
	) {
		for (_asset_id, _weight) in assets {
			// if let Err(_) = T::Fungibles::update_system_token_weight(asset_id, weight) {
			// 	TODO: Handle Error
			// }
		}
	}
}

impl<T: Config> TaaV for Pallet<T> {
	type Error = ();

	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
		cumulus_pallet_parachain_system::Pallet::<T>::handle_vote(bytes.clone()); 
		Ok(())
	}
}
