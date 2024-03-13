use frame_support::{
	pallet_prelude::*,
	traits::tokens::fungibles::{Inspect, InspectSystemToken},
	DefaultNoBound,
};
use frame_system::{ensure_root, pallet_prelude::*};
use log;
use pallet_validator_election::PotInterface;
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
		/// Type that interacts with local asset
		type Fungibles: InspectSystemToken<Self::AccountId>;
		/// Updating vote type
		type Voting: PotInterface<Self::AccountId>;
		/// Managing System Token
		type SystemTokenInterface: SystemTokenInterface<
			SystemTokenAssetIdOf<T>,
			SystemTokenBalanceOf<T>,
			VoteWeightOf<T>,
			RemoteAssetMetadata<SystemTokenAssetIdOf<T>, SystemTokenBalanceOf<T>>,
		>;
		/// Type that delivers XCM messages
		type XcmRouter: SendXcm;
	}

	/// System configuration for `InfraRelay` Runtime
	#[pallet::storage]
	#[pallet::getter(fn active_system_config)]
	pub type ActiveSystemConfig<T: Config> =
		StorageValue<_, SystemTokenConfig<SystemTokenWeightOf<T>>, ValueQuery>;

	/// Relay Chain's tx fee rate
	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenBalanceOf<T>>;

	/// Relay Chain's Runtime state
	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	/// Relay Chain's fee for each extrinsic
	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted {
			who: T::AccountId,
			system_token_id: SystemTokenAssetIdOf<T>,
			vote_weight: VoteWeightOf<T>,
		},
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: SystemTokenBalanceOf<T> },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: SystemTokenAssetIdOf<T> },
		/// Bootstrap has been ended by Relay-chain governance.
		BootstrapEnded,
		/// Infra configuration has been udpated
		InfraConfigUpdated { new: SystemTokenConfig<SystemTokenWeightOf<T>> },
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
		pub system_token_config: SystemTokenConfig<SystemTokenWeightOf<T>>,
		pub _phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.system_token_config.panic_if_not_validated();
			ActiveSystemConfig::<T>::put(self.system_token_config.clone());
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
		pub fn update_system_token_config(
			origin: OriginFor<T>,
			system_token_config: SystemTokenConfig<SystemTokenWeightOf<T>>,
		) -> DispatchResult {
			// TODO: Need Scheduler for upadating InfraSystemConfig
			// TODO: Base configuration for InfraRelaychain has changed. Needs to update all
			// parachains' config.
			ensure_root(origin)?;
			ActiveSystemConfig::<T>::put(system_token_config.clone());
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
