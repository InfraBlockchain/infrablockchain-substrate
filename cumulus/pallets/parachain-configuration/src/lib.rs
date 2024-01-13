
#[cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::traits::tokens::{Balance, AssetId};
use frame_system::pallet_prelude::*;
use cumulus_pallet_xcm::{ensure_relay, Origin};
use sp_runtime::types::{SystemTokenWeight, SystemTokenBalance, SystemTokenAssetId, ExtrinsicMetadata, Mode, RuntimeConfigProvider};

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Runtime Origin for the System Token pallet.
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<Origin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type BaseWeight<T: Config> = StorageValue<_, SystemTokenWeight, OptionQuery>;

	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenWeight, OptionQuery>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance<T>, OptionQuery>;

	#[pallet::storage]
	pub type SystemToken<T: Config> = StorageMap<_, Blake2_128Concat, SystemTokenAssetId, SystemTokenDetails, OptionQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: SystemTokenBalance<T> },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: SystemTokenAssetId },
		/// Bootstrap has been ended by Relay-chain governance. 
		BootstrapEnded
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Mode of Runtime cannot be changed(e.g SystemTokenMissing)
		NotAllowedToChangeState,
		/// System Token is not registered
		SystemTokenMissing,
		/// Base System Token weight has not been set 
		BaseWeightMissing,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Base system token weight configuration will be set by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(0)]
		pub fn set_base_weight(origin: OriginFor<T>) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			BaseWeight::<T>::put(T::BaseWeight::get());
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
			fee: SystemTokenBalance<T>,
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
		pub fn set_fee_rate(
			origin: OriginFor<T>,
			fee_rate: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			FeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set runtime state configuration for this parachain by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
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
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
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
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
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
		) -> DispachResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			// Assets::create()
			Ok(())
		}

		#[pallet::call_index(8)]
		pub fn deregister(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
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

impl<T: Config> RuntimeConfigProvider for Pallet<T> {
	type Balance = SystemTokenBalance<T>;
	fn fee_rate() -> SystemTokenWeight {
		let base_weight = BaseWeight::<T>::get().ok_or(Error::<T>::BaseWeightMissing)?;
		FeeRate::<T>::get().map_or(base_weight, |x| x)
	}
	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance<T>> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}