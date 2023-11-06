//! Benchmarking setup for pallet-asset-registry

use super::*;

#[allow(unused)]
use crate::Pallet as AssetLink;
use frame_benchmarking::benchmarks;
use frame_support::{assert_ok, traits::fungibles::Inspect};
use frame_system::RawOrigin;
use xcm::opaque::latest::{
	Junction::{GeneralIndex, PalletInstance, Parachain},
	Junctions, MultiLocation,
};

pub const LOCAL_ASSET_ID: u32 = 10;

benchmarks! {
	where_clause {
		where
			T::Assets: Inspect<<T as frame_system::Config>::AccountId, AssetId = u32>,
	}

	link_system_token {
		let asset_multi_location = MultiLocation {
			parents: 1,
			interior: Junctions::X3(Parachain(Default::default()), PalletInstance(Default::default()), GeneralIndex(Default::default()))
		};

	}: _(RawOrigin::Root, LOCAL_ASSET_ID, asset_multi_location.clone())
	verify {
		let read_asset_multi_location = AssetLink::<T>::asset_id_multilocation(LOCAL_ASSET_ID)
			.expect("error reading AssetIdMultiLocation");
		assert_eq!(read_asset_multi_location, asset_multi_location);
	}

	unlink_system_token {
		let asset_multi_location = MultiLocation {
			parents: 1,
			interior: Junctions::X3(Parachain(Default::default()), PalletInstance(Default::default()), GeneralIndex(Default::default()))
		};

		assert_ok!(AssetLink::<T>::link_system_token(RawOrigin::Root.into(), LOCAL_ASSET_ID, asset_multi_location.clone()));
		let read_asset_multi_location = AssetLink::<T>::asset_id_multilocation(LOCAL_ASSET_ID)
			.expect("error reading AssetIdMultiLocation");
		assert_eq!(read_asset_multi_location, asset_multi_location);

	}: _(RawOrigin::Root, LOCAL_ASSET_ID)
	verify {
		assert_eq!(AssetLink::<T>::asset_id_multilocation(LOCAL_ASSET_ID), None);
	}

	impl_benchmark_test_suite!(AssetLink, crate::mock::new_test_ext(), crate::mock::Test);
}
