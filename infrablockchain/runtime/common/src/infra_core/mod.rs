
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_system::ensure_root;
use parity_scale_codec::Encode;
use sp_runtime::types::{token::*, fee::*, vote::*, infra_core::*};
use sp_std::vec::Vec;
use pallet_validator_election::VotingInterface;
use xcm::latest::prelude::*;

mod impls;
mod types;

pub use pallet::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct SystemTokenDetails {
	pub weight: SystemTokenWeight,
	pub status: SystemTokenStatus
}

impl SystemTokenDetails {
	pub fn set_weight(&mut self, weight: SystemTokenWeight) {
		self.weight = weight;
	}

	pub fn set_status(&mut self, status: SystemTokenStatus) {
		self.status = status;
	}
}

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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Updating vote type
		type VotingInterface: VotingInterface<Self>;
		/// Managing System Token
		type SystemTokenInterface: SystemTokenInterface;
		/// Type that interacts with local asset
		type LocalAssetManager: LocalAssetManager<Self::AccountId>;
		/// Type that delivers XCM messages
		type XcmRouter: SendXcm;
        /// Base system token weight for InfraBlockchain
        #[pallet::constant]
        type BaseWeight: Get<SystemTokenWeight>;
    }

	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenWeight, OptionQuery>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

    #[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance, OptionQuery>;
    
    #[pallet::storage]
	pub type SystemToken<T: Config> = StorageMap<_, Blake2_128Concat, SystemTokenAssetId, SystemTokenDetails, OptionQuery>;

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted { who: VoteAccountId, system_token_id: SystemTokenId, vote_weight: VoteWeight },
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: SystemTokenBalance },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: SystemTokenAssetId },
		/// Bootstrap has been ended by Relay-chain governance. 
		BootstrapEnded
	}

    #[pallet::error]
	pub enum Error<T> {
		NotAllowedToChangeState,
		SystemTokenMissing,
		NotSystemToken,
		OnUpdateVote, 
	}

    #[pallet::call]
	impl<T: Config> Pallet<T> {

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
		pub fn set_fee_rate(
			origin: OriginFor<T>,
			fee_rate: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			FeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set runtime state configuration for this parachain by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Self::do_set_runtime_state()?;
			Ok(())
		}

		/// System Token weight configuration is set by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(5)]
		pub fn set_system_token_weight(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut system_token_detail = SystemToken::<T>::get(&asset_id).ok_or(Error::<T>::SystemTokenMissing)?;
			system_token_detail.set_weight(system_token_weight);
			SystemToken::<T>::insert(&asset_id, system_token_detail);
			Self::deposit_event(Event::<T>::SystemTokenWeightUpdated { asset_id });
			Ok(())
		}

		/// Register System Token for Cumulus-based parachain Runtime.
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(6)]
		pub fn register(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight
		) -> DispatchResult {
			ensure_root(origin)?;
			// Assets::promote()
			Ok(())
		}
		
		/// Discription 
		/// Asset which referes to `wrapped` System Token will be created by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(7)]
		pub fn create(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight
		) -> DispatchResult {
			ensure_root(origin)?;
			// Assets::create()
			Ok(())
		}

		#[pallet::call_index(8)]
		pub fn deregister(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;
			// Assets::demote()
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_set_runtime_state() -> DispatchResult {
		if RuntimeState::<T>::get() == Mode::Normal {
			return Ok(())
		}
		ensure!(SystemToken::<T>::iter_keys().count() != 0, Error::<T>::NotAllowedToChangeState);
		// ToDo: Check whether a parachain has enough system token to pay
		RuntimeState::<T>::put(Mode::Normal);
		Self::deposit_event(Event::<T>::BootstrapEnded);
		Ok(())
	}
}