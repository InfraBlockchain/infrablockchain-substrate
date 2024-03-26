#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use cumulus_pallet_xcm::{ensure_relay, Origin};
use cumulus_primitives_core::{UpdateRCConfig, ParaId};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::{
		Inspect, InspectSystemToken, InspectSystemTokenMetadata, ManageSystemToken,
	}, tokens::SystemTokenId},
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
pub type SystemTokenOriginIdOf<T> = <<T as Config>::SystemTokenId as SystemTokenId>::OriginId;

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
		/// Type of SystemTokenId used in InfraBlockchain
		type SystemTokenId: SystemTokenId;
		/// Type that interacts with local asset
		type Fungibles: InspectSystemToken<Self::AccountId>
			+ InspectSystemTokenMetadata<Self::AccountId>
			+ ManageSystemToken<Self::AccountId>;
		/// Active request period for registering System Token
		#[pallet::constant]
		type ActiveRequestPeriod: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId>;

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
	pub type RequestQueue<T: Config> = StorageValue<_, SystemTokenAssetIdOf<T>>;

	#[pallet::storage]
	pub type ActiveRequestStatus<T: Config> = StorageValue<_, RequestStatus<BlockNumberFor<T>>>;

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
		AdminUpdated { who: T::AccountId },
		/// System Token registration has been requested
		RegisterRequested { asset_id: SystemTokenAssetIdOf<T>, exp: BlockNumberFor<T> },
		/// Wrapped local asset has been created
		WrappedCreated { asset_id: SystemTokenAssetIdOf<T> },
		/// System Token has been suspended by Relay-chain governance
		Suspended { asset_id: SystemTokenAssetIdOf<T> },
		/// System Token has been unsuspended by Relay-chain governance
		Unsuspended { asset_id: SystemTokenAssetIdOf<T> },
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
		/// Error occured while suspending System Token
		ErrorSuspendSystemToken,
		/// Error occured while unsuspending System Token
		ErrorUnsuspendSystemToken,
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
		BadRequest,
		/// Error occured while converting System Token ID
		ErrorConvertToSystemTokenId,
		/// System Token is not native asset
		NotNativeAsset
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> frame_support::weights::Weight {
			if let Some(status) = ActiveRequestStatus::<T>::get() {
				if status.is_expired(n) {
					ActiveRequestStatus::<T>::kill();
					RequestQueue::<T>::kill();
				}
				T::DbWeight::get().reads_writes(1, 2)
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> 
	where
		T::SystemTokenId: TryFrom<SystemTokenAssetIdOf<T>> + Into<SystemTokenAssetIdOf<T>>,
		SystemTokenOriginIdOf<T>: From<ParaId>
	{
		/// Priviliged origin governed by Relay-chain
		///
		/// It can call extrinsic which is not allowed to call by other origin(e.g
		/// `request_register_system_token`)
		#[pallet::call_index(0)]
		pub fn set_admin(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			Admin::<T>::put(&who);
			Self::deposit_event(Event::<T>::AdminUpdated { who });
			Ok(())
		}

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
			original: SystemTokenAssetIdOf<T>,
			system_token_weight: SystemTokenWeightOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::Fungibles::register(&original, system_token_weight)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Self::clear_request();
			Self::deposit_event(Event::<T>::Registered { asset_id: original });
			Ok(())
		}

		/// Discription
		/// Asset which referes to `wrapped` System Token will be created by Relay-chain governance
		///
		/// Parameters
		/// - `asset_id`: AssetId of `wrapped` System Token
		/// - `system_token_id`: SystemTokenId of wrapped `original` System Token
		/// - `system_token_weight`: Weight of `wrapped` System Token. Need for `AssetLink`
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(5)]
		pub fn create_wrapped(
			origin: OriginFor<T>,
			owner: T::AccountId,
			wrapped_original: SystemTokenAssetIdOf<T>,
			currency_type: Fiat,
			min_balance: SystemTokenBalanceOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			system_token_weight: SystemTokenWeightOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			log::info!("😏😏😏😏 Wrapping {:?}", wrapped_original);
			T::Fungibles::touch(
				owner,
				wrapped_original.clone(),
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight,
			)
			.map_err(|_| {
				log::info!("😡😡😡😡😡 Error on creating wrapped local asset");
				Error::<T>::ErrorCreateWrappedLocalAsset
			})?;
			Self::deposit_event(Event::<T>::WrappedCreated { asset_id: wrapped_original });
			Ok(())
		}

		#[pallet::call_index(6)]
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::Fungibles::deregister(&asset_id)
				.map_err(|_| Error::<T>::ErrorDeregisterSystemToken)?;
			Self::deposit_event(Event::<T>::Deregistered { asset_id });
			Ok(())
		}

		#[pallet::call_index(7)]
		pub fn suspend_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::Fungibles::suspend(&asset_id)
				.map_err(|_| Error::<T>::ErrorSuspendSystemToken)?;
			Self::deposit_event(Event::<T>::Suspended { asset_id });
			Ok(())
		}

		#[pallet::call_index(8)]
		pub fn unsuspend_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetIdOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			T::Fungibles::unsuspend(&asset_id)
				.map_err(|_| Error::<T>::ErrorUnsuspendSystemToken)?;
			Self::deposit_event(Event::<T>::Unsuspended { asset_id });
			Ok(())
		}

		/// Request to register System Token
		///
		/// If succeed, request will be queued in `CurrentRequest`
		#[pallet::call_index(9)]
		pub fn request_register_system_token(
			origin: OriginFor<T>,
			original: SystemTokenAssetIdOf<T>,
			currency_type: Fiat,
		) -> DispatchResult {
			if let Some(acc) = ensure_signed_or_root(origin)? {
				ensure!(Admin::<T>::get() == Some(acc), Error::<T>::NoPermission);
			}
			let exp = Self::do_request(&original, currency_type)?;
			Self::deposit_event(Event::<T>::RegisterRequested { asset_id: original, exp });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> 
where
	T::SystemTokenId: TryFrom<SystemTokenAssetIdOf<T>> + Into<SystemTokenAssetIdOf<T>>,
	SystemTokenOriginIdOf<T>: From<ParaId>
{
	/// Put requested _asset_metadata_  on `CurrentRequest` and calculate expired block numbe
	fn do_request(original: &SystemTokenAssetIdOf<T>, currency_type: Fiat) -> Result<BlockNumberFor<T>, DispatchError> {
		let current = <frame_system::Pallet<T>>::block_number();
		T::Fungibles::request_register(original, currency_type)
			.map_err(|_| Error::<T>::ErrorOnRequestRegister)?;
		let mut system_token_metadata = T::Fungibles::system_token_metadata(original)
			.map_err(|_| Error::<T>::ErrorOnGetMetadata)?;
		Self::check_valid_register(&mut system_token_metadata, original)?;
		<cumulus_pallet_parachain_system::Pallet<T>>::relay_request_asset(
			system_token_metadata.encode(),
		);
		let exp = current.saturating_add(T::ActiveRequestPeriod::get());
		let request_status = RequestStatus::default_status(exp);
		ActiveRequestStatus::<T>::put(request_status);
		RequestQueue::<T>::put(original);
		Ok(exp)
	}

	fn check_valid_register(asset_metadata: &mut RemoteAssetMetadata<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>, asset: &SystemTokenAssetIdOf<T>) -> Result<(), DispatchError> {
		let mut is_valid: bool = true;
		let system_token_id: T::SystemTokenId = asset.clone().try_into().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
		let origin_id = <<T as cumulus_pallet_parachain_system::Config>::SelfParaId>::get();
		let (maybe_origin_id, pallet_id, asset_id) = system_token_id.id().map_err(|_| Error::<T>::ErrorConvertToSystemTokenId)?;
		ensure!(maybe_origin_id.is_none(), Error::<T>::BadRequest);
		let system_token_id = T::SystemTokenId::convert_back(Some(origin_id.into()), pallet_id, asset_id);
		asset_metadata.set_asset_id(system_token_id.into());
		if let Some(status) = ActiveRequestStatus::<T>::get() {
			if !status.is_expired(<frame_system::Pallet<T>>::block_number()) {
				ActiveRequestStatus::<T>::kill();
				RequestQueue::<T>::kill();
			} else {
				is_valid = false;
			}
		} 
		ensure!(is_valid, Error::<T>::BadRequest);
		Ok(())
	}

	fn clear_request() {
		ActiveRequestStatus::<T>::kill();
		RequestQueue::<T>::kill();
	}
}

impl<T: Config> RuntimeConfigProvider<SystemTokenBalanceOf<T>> for Pallet<T>
where
	SystemTokenBalanceOf<T>: From<u128>,
{
	type Error = DispatchError;

	fn system_config() -> Result<SystemConfig, Self::Error> {
		Ok(RCSystemConfig::<T>::get().ok_or(Error::<T>::NotInitiated)?)
	}

	fn para_fee_rate() -> Result<SystemTokenBalanceOf<T>, Self::Error> {
		let base_weight = RCSystemConfig::<T>::get()
			.ok_or(Error::<T>::NotInitiated)?
			.base_system_token_detail
			.base_weight;
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
		for (asset_id, weight) in assets {
			if let Err(_) = T::Fungibles::update_system_token_weight(&asset_id, weight) {
				log::error!("❌❌❌ Error on updating System Token Weight, {:?}", asset_id)
			}
		}
	}
}
