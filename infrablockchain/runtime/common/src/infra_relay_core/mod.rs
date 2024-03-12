use frame_support::{
	pallet_prelude::*,
	traits::tokens::fungibles::{Inspect, InspectSystemToken},
	DefaultNoBound,
};
use frame_system::{ensure_root, pallet_prelude::*};
use log;
use pallet_validator_election::VotingInterface;
use parity_scale_codec::Encode;
use primitives::well_known_keys;
use runtime_parachains::SystemTokenInterface;
use softfloat::F64;
use sp_runtime::types::{fee::*, infra_core::*, token::*, vote::*};
use sp_std::vec::Vec;
use xcm::latest::prelude::*;

mod impls;
mod types;

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type of weight for voting
		type VoteWeight: Into<F64>;
		/// Updating vote type
		type VotingInterface: VotingInterface<Self>;
		/// Managing System Token
		type SystemTokenInterface: SystemTokenInterface;
		/// Type that interacts with local asset
		type Fungibles: InspectSystemToken<Self::AccountId>;
		/// Type that links asset with System Token
		type AssetLink: AssetLinkInterface<SystemTokenAssetId>;
		/// Type that delivers XCM messages
		type XcmRouter: SendXcm;
	}

	/// System configuration for `InfraRelay` Runtime
	#[pallet::storage]
	#[pallet::getter(fn active_system_config)]
	pub type ActiveSystemConfig<T: Config> = StorageValue<_, SystemTokenConfig, ValueQuery>;

	/// Relay Chain's tx fee rate
	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenWeight>;

	/// Relay Chain's Runtime state
	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	/// Relay Chain's fee for each extrinsic
	#[pallet::storage]
	pub type FeeTable<T: Config> = StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted {
			who: VoteAccountId,
			system_token_id: SystemTokenId,
			vote_weight: VoteWeight,
		},
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated {
			extrinsic_metadata: ExtrinsicMetadata,
			fee: SystemTokenBalance,
		},
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated {
			asset_id: SystemTokenAssetId,
		},
		/// Bootstrap has been ended by Relay-chain governance.
		BootstrapEnded,
		/// Asset is linked since it has registered as System Token by Relay-chain governance
		AssetLinked {
			asset_id: SystemTokenAssetId,
			multi_loc: MultiLocation,
		},
		/// Asset is unlinked by Relay-chain governance
		AssetUnlinked {
			asset_id: SystemTokenAssetId,
		},
		/// Infra configuration has been udpated
		InfraConfigUpdated {
			new: InfraSystemConfig,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Current Runtime state is not ready to change
		NotAllowedToChangeState,
		/// Error occured while registering System Token
		ErrorRegisterSystemToken,
		/// Error occured while updating weight of System Token
		ErrorUpdateWeight,
		/// Error occured while creating wrapped local asset
		ErrorCreateWrappedLocal,
		/// Error occured while linking asset
		ErrorLinkAsset,
		/// Error occured while deregistering asset
		ErrorDeregisterSystemToken,
		/// Module has not been initialized
		NotInitialized,
		/// Error occured while decoding
		ErrorDecode,
	}

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub system_config: InfraSystemConfig,
		pub _phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.system_config.panic_if_not_validated();
			ActiveSystemConfig::<T>::put(self.system_config.clone());
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert_eq!(
				&ActiveSystemConfig::<T>::hashed_key(),
				well_known_keys::SYSTEM_CONFIG,
				"`well_known_keys::SYSTEM_CONFIG` doesn't match key of `ActiveConfig`! Make sure that the name of the\
				 configuration pallet is `Configuration` in the runtime!",
			);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		pub fn update_infra_system_config(
			origin: OriginFor<T>,
			infra_system_config: InfraSystemConfig,
		) -> DispatchResult {
			// TODO: Need Scheduler for upadating InfraSystemConfig
			// TODO: Base configuration for InfraRelaychain has changed. Needs to update all
			// parachains' config.
			ensure_root(origin)?;
			ActiveSystemConfig::<T>::put(infra_system_config.clone());
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
			ensure_root(origin)?;
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
		pub fn set_fee_rate(origin: OriginFor<T>, fee_rate: SystemTokenWeight) -> DispatchResult {
			ensure_root(origin)?;
			FeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set runtime state configuration
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn update_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Self::do_update_runtime_state();
			Self::deposit_event(Event::<T>::BootstrapEnded);
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(4)]
		pub fn set_system_token_weight(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::update_system_token_weight(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorUpdateWeight)?;
			Self::deposit_event(Event::<T>::SystemTokenWeightUpdated { asset_id });
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(5)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::promote(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
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
			asset_link_parent: u8,
			original: SystemTokenId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::create_wrapped_local(
				asset_id,
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight,
			)
			.map_err(|_| Error::<T>::ErrorCreateWrappedLocal)?;
			T::AssetLink::link(&asset_id, asset_link_parent, original)
				.map_err(|_| Error::<T>::ErrorLinkAsset)?;
			Ok(())
		}

		#[pallet::call_index(7)]
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::demote(asset_id)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_update_runtime_state() {
		if RuntimeState::<T>::get() == Mode::Normal {
			return
		}
		// TODO-1: Check whether it is allowed to change `Normal` state
		// ToDo-2: Check whether a parachain has enough system token to pay
		RuntimeState::<T>::put(Mode::Normal);
	}
}
