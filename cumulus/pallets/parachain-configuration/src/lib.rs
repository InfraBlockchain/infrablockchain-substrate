
#[cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::traits::tokens::{Balance, AssetId};
use frame_system::pallet_prelude::*;
use cumulus_pallet_xcm::{ensure_relay, Origin};
use sp_runtime::types::{SystemTokenWeight, ExtrinsicMetadata, Mode, RuntimeConfigProvider};

pub use pallet::*;

type AssetBalanceOf<T> = <<T as Config>::AssetsInterface as AssetsInterface>::Balance;
type AssetIdOf<T> = <<T as Config>::AssetsInterface as AssetsInterface>::AssetId;

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
		/// Types that implement the `AssetsInterface` trait.
		// type AssetsInterface: AssetsInterface;
		/// The base weight of system token for this Runtime.
		#[pallet::constant]
		type BaseWeight: Get<SystemTokenWeight>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub fee_table: Option<Vec<(ExtrinsicMetadata, AssetBalanceOf<T>)>>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			FeeRate::<T>::put(T::BaseWeight::get());
			if let Some(f_t) = self.fee_table.clone() {
				for (extrinsic_metadata, fee) in f_t {
					FeeTable::<T>::insert(&extrinsic_metadata, fee);
				}
			}
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenWeight, OptionQuery>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, AssetBalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type SystemToken<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, SystemTokenDetails, OptionQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated { extrinsic_metadata: ExtrinsicMetadata, fee: AssetBalanceOf<T> },
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated { asset_id: AssetIdOf<T> },
		/// Bootstrap has been ended by Relay-chain governance. 
		BootstrapEnded
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Mode of Runtime cannot be changed(e.g SystemTokenNotExist)
		NotAllowedToChangeState,
		/// System Token is not registered
		SystemTokenNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register System Token for Cumulus-based parachain Runtime.
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			system_token_weight: SystemTokenWeight
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			Ok(())
		}

		/// Set policy for fee table of Runtime by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(1)]
		pub fn set_fee_table(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			call_name: Vec<u8>,
			fee: AssetBalanceOf<T>,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			let extrinsic_metadata = ExtrinsicMetadata::new(pallet_name, call_name);
			FeeTable::<T>::insert(&extrinsic_metadata, fee);
			Self::deposit_event(Event::<T>::FeeTableUpdated { extrinsic_metadata, fee });
			Ok(())
		}

		#[pallet::call_index(2)]
		pub fn set_fee_rate(
			origin: OriginFor<T>,
			fee_rate: SystemTokenWeight,
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			FeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set policy for runtime state of Runtime by Relay-chain governance
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			Self::do_set_runtime_state()?;
			Ok(())
		}
		
		/// Set policy of System Token weight 
		/// 
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(4)]
		pub fn set_system_token_weight(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			system_token_weight: SystemTokenWeight
		) -> DispatchResult {
			ensure_relay(<T as Config>::RuntimeOrigin::from(origin))?;
			let mut system_token_detail = SystemToken::<T>::get(&asset_id).ok_or(Error::<T>::SystemTokenNotExist)?;
			system_token_detail.set_weight(system_token_weight);
			SystemToken::<T>::insert(&asset_id, system_token_detail);
			Self::deposit_event(Event::<T>::SystemTokenWeightUpdated { asset_id });
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

impl<T: Config> GlobalConfigProvider for Pallet<T> {
	fn fee_rate() -> SystemTokenWeight {
		FeeRate::<T>::get().map_or(T::BaseWeight::get(), |x| x);
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<AssetBalanceOf<Self>> {
		FeeTable::<T>::get(&ext)
	}

	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

impl<T: Config> RuntimeConfigProvider for Pallet<T> {
	type Balance = AssetBalanceOf<T>;
	fn fee_rate() -> SystemTokenWeight {
		FeeRate::<T>::get().map_or(T::BaseWeight::get(), |x| x)
	}
	fn fee_for(ext: ExtrinsicMetadata) -> Option<AssetBalanceOf<T>> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

pub trait AssetsInterface {
	type AssetId: AssetId;
	type Balance: Balance;
}

impl AssetsInterface for () {
	type AssetId = u32;
	type Balance = u128;
}