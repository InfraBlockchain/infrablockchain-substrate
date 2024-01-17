#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{pallet_prelude::*, traits::tokens::fungibles::Inspect};
use sp_runtime::types::token::*;
use xcm::latest::prelude::*;
use xcm_primitives::AssetMultiLocationGetter;

pub use pallet::*;

pub type AssetIdOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Assets: Inspect<Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_id_multilocation)]
	pub type AssetIdMultiLocation<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, MultiLocation, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn asset_multilocation_id)]
	pub type AssetMultiLocationId<T: Config> =
		StorageMap<_, Twox64Concat, MultiLocation, AssetIdOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetLinked { asset_id: AssetIdOf<T>, asset_multi_location: MultiLocation },
		AssetUnlinked { asset_id: AssetIdOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The Asset ID is already registered
		AssetAlreadyLinked,
		/// The Asset ID does not exist
		AssetDoesNotExist,
		/// The Asset ID is not registered
		AssetIsNotLinked,
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

impl<T: Config> AssetLinkInterface<AssetIdOf<T>> for Pallet<T>
where
	AssetIdOf<T>: From<SystemTokenAssetId>,
{
	type Error = DispatchError;

	fn link(asset_id: &AssetIdOf<T>, parents: u8, original: SystemTokenId) -> DispatchResult {
		ensure!(T::Assets::asset_exists(asset_id.clone()), Error::<T>::AssetDoesNotExist);
		ensure!(
			!AssetIdMultiLocation::<T>::contains_key(&asset_id),
			Error::<T>::AssetAlreadyLinked
		);
		let SystemTokenId { para_id, pallet_id, asset_id } = original;
		let asset_multi_location = MultiLocation {
			parents,
			interior: Junctions::X3(
				Parachain(para_id),
				PalletInstance(pallet_id),
				GeneralIndex(asset_id as u128),
			),
		};
		let id: AssetIdOf<T> = asset_id.into();
		AssetIdMultiLocation::<T>::insert(&id, &asset_multi_location);
		AssetMultiLocationId::<T>::insert(&asset_multi_location, id.clone());

		Self::deposit_event(Event::AssetLinked { asset_id: id, asset_multi_location });

		Ok(())
	}

	fn unlink(asset_id: &AssetIdOf<T>) -> Result<(), Self::Error> {
		let asset_multi_location =
			AssetIdMultiLocation::<T>::get(asset_id).ok_or(Error::<T>::AssetIsNotLinked)?;

		AssetIdMultiLocation::<T>::remove(asset_id);
		AssetMultiLocationId::<T>::remove(&asset_multi_location);

		Self::deposit_event(Event::AssetUnlinked { asset_id: asset_id.clone() });

		Ok(())
	}
}
