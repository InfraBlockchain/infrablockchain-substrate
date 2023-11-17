#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use pallet::*;

pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::tokens::fungibles::Inspect,
	};
	use frame_system::pallet_prelude::*;
	use pallet_assets::AssetLinkInterface;

	use sp_runtime::types::SystemTokenId;
	use xcm::latest::{
		Junction::{GeneralIndex, PalletInstance, Parachain},
		Junctions, MultiLocation,
	};
	use xcm_primitives::AssetMultiLocationGetter;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type AssetIdOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ReserveAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type Assets: Inspect<Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_id_multilocation)]
	pub type AssetIdMultiLocation<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, MultiLocation>;

	#[pallet::storage]
	#[pallet::getter(fn asset_multilocation_id)]
	pub type AssetMultiLocationId<T: Config> =
		StorageMap<_, Blake2_128Concat, MultiLocation, AssetIdOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetLinked { asset_id: AssetIdOf<T>, asset_multi_location: MultiLocation },
		AssetUnlinked { asset_id: AssetIdOf<T>, asset_multi_location: MultiLocation },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The Asset ID is already registered
		AssetAlreadyLinked,
		/// The Asset ID does not exist
		AssetDoesNotExist,
		/// The Asset ID is not registered
		AssetIsNotLinked,
		/// Invalid MultiLocation
		WrongMultiLocation,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::link_system_token())]
		pub fn link_system_token(
			origin: OriginFor<T>,
			parents: u8,
			asset_id: AssetIdOf<T>,
			system_token_id: SystemTokenId,
		) -> DispatchResult {
			T::ReserveAssetModifierOrigin::ensure_origin(origin)?;

			// verify asset exists on pallet-assets
			ensure!(Self::asset_exists(&asset_id), Error::<T>::AssetDoesNotExist);

			// verify asset is not yet registered
			ensure!(
				!AssetIdMultiLocation::<T>::contains_key(&asset_id),
				Error::<T>::AssetAlreadyLinked
			);

			let asset_multi_location = MultiLocation {
				parents,
				interior: Junctions::X3(
					Parachain(system_token_id.para_id),
					PalletInstance(system_token_id.pallet_id as u8),
					GeneralIndex(system_token_id.asset_id as u128),
				),
			};

			// verify MultiLocation is valid
			let junctions_multi_location_ok = matches!(
				asset_multi_location.interior,
				Junctions::X3(Parachain(_), PalletInstance(_), GeneralIndex(_))
			);

			ensure!(junctions_multi_location_ok, Error::<T>::WrongMultiLocation);

			// register asset
			AssetIdMultiLocation::<T>::insert(&asset_id, &asset_multi_location);
			AssetMultiLocationId::<T>::insert(&asset_multi_location, asset_id.clone());

			Self::deposit_event(Event::AssetLinked { asset_id, asset_multi_location });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unlink_system_token())]
		pub fn unlink_system_token(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
			T::ReserveAssetModifierOrigin::ensure_origin(origin)?;

			// verify asset is registered
			let asset_multi_location =
				AssetIdMultiLocation::<T>::get(&asset_id).ok_or(Error::<T>::AssetIsNotLinked)?;

			// unregister asset
			AssetIdMultiLocation::<T>::remove(&asset_id);
			AssetMultiLocationId::<T>::remove(&asset_multi_location);

			Self::deposit_event(Event::AssetUnlinked { asset_id, asset_multi_location });
			Ok(())
		}
	}

	impl<T: Config> AssetMultiLocationGetter<AssetIdOf<T>> for Pallet<T> {
		fn get_asset_multi_location(asset_id: AssetIdOf<T>) -> Option<MultiLocation> {
			AssetIdMultiLocation::<T>::get(asset_id)
		}

		fn get_asset_id(asset_type: MultiLocation) -> Option<AssetIdOf<T>> {
			AssetMultiLocationId::<T>::get(asset_type)
		}
	}

	impl<T: Config> Pallet<T> {
		// check if the asset exists
		fn asset_exists(asset_id: &AssetIdOf<T>) -> bool {
			T::Assets::asset_exists(asset_id.clone())
		}
	}

	impl<T> AssetLinkInterface<AssetIdOf<T>> for Pallet<T>
	where
		T: Config,
	{
		fn link_system_token(
			parents: u8,
			asset_id: &AssetIdOf<T>,
			system_token_id: SystemTokenId,
		) -> DispatchResult {
			// verify asset exists on pallet-assets
			ensure!(Self::asset_exists(asset_id), Error::<T>::AssetDoesNotExist);

			// verify asset is not yet registered
			ensure!(
				!AssetIdMultiLocation::<T>::contains_key(asset_id),
				Error::<T>::AssetAlreadyLinked
			);

			let asset_multi_location = MultiLocation {
				parents,
				interior: Junctions::X3(
					Parachain(system_token_id.para_id),
					PalletInstance(system_token_id.pallet_id),
					GeneralIndex(system_token_id.asset_id as u128),
				),
			};

			// verify MultiLocation is valid
			let junctions_multi_location_ok = matches!(
				asset_multi_location.interior,
				Junctions::X3(Parachain(_), PalletInstance(_), GeneralIndex(_))
			);

			ensure!(junctions_multi_location_ok, Error::<T>::WrongMultiLocation);

			// register asset
			AssetIdMultiLocation::<T>::insert(asset_id, &asset_multi_location);
			AssetMultiLocationId::<T>::insert(&asset_multi_location, asset_id);

			Self::deposit_event(Event::AssetLinked {
				asset_id: asset_id.clone(),
				asset_multi_location,
			});

			Ok(())
		}

		fn unlink_system_token(asset_id: &AssetIdOf<T>) -> DispatchResult {
			// verify asset is registered
			let asset_multi_location =
				AssetIdMultiLocation::<T>::get(asset_id).ok_or(Error::<T>::AssetIsNotLinked)?;

			// unregister asset
			AssetIdMultiLocation::<T>::remove(asset_id);
			AssetMultiLocationId::<T>::remove(&asset_multi_location);

			Self::deposit_event(Event::AssetUnlinked {
				asset_id: asset_id.clone(),
				asset_multi_location,
			});
			Ok(())
		}
	}
}
