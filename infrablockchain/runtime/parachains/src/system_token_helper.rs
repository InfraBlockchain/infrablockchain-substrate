use crate::{
	configuration,
	dmp::{self, Config},
};
use frame_support::{
	pallet_prelude::{DispatchResult, Weight},
	traits::{fungibles::roles::Inspect, OriginTrait},
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::Encode;
use sp_runtime::{
	traits::AccountIdConversion,
	types::{PalletId, ParaId},
};
use sp_std::{boxed::Box, vec, vec::Vec};

use xcm::{
	opaque::{
		latest::prelude::*,
		lts::{AssetId::Concrete, Fungibility::Fungible, Junction, MultiAsset, MultiLocation},
		VersionedXcm,
	},
	v2::OriginKind,
};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

const REF_WEIGHT: u64 = 1_000_000_000;
const PROOF_WEIGHT: u64 = 50_000;

pub fn root_account<T: frame_system::Config>() -> T::AccountId {
	frame_support::PalletId(*b"infra/rt").into_account_truncating()
}

pub fn sovereign_account<T: frame_system::Config>() -> T::AccountId {
	frame_support::PalletId(*b"infrapid").into_account_truncating()
}

pub fn inspect_account_and_check_is_owner<T: pallet_assets::Config>(asset_id: &T::AssetId) -> bool {
	let default_acc: T::AccountId = frame_support::PalletId(*b"infrapid").into_account_truncating();
	let root_acc = root_account::<T>();
	root_acc ==
		pallet_assets::Pallet::<T>::owner(asset_id.clone()).map_or(default_acc.clone(), |a| a) &&
		root_acc ==
			pallet_assets::Pallet::<T>::issuer(asset_id.clone())
				.map_or(default_acc.clone(), |a| a) &&
		root_acc ==
			pallet_assets::Pallet::<T>::admin(asset_id.clone())
				.map_or(default_acc.clone(), |a| a) &&
		root_acc ==
			pallet_assets::Pallet::<T>::freezer(asset_id.clone()).map_or(default_acc, |a| a)
}

fn encode_pallet_call(pallet_id: PalletId, mut encoded_call: Vec<u8>) -> Vec<u8> {
	let mut encoded: Vec<u8> = [pallet_id].into();
	encoded.append(&mut encoded_call);
	encoded
}

fn transact_xcm(
	fees: MultiAsset,
	origin_kind: Option<OriginKind>,
	require_weight_at_most: Option<Weight>,
	encoded_call: Vec<u8>,
) -> Vec<u8> {
	VersionedXcm::from(Xcm(vec![
		BuyExecution { fees: fees.into(), weight_limit: WeightLimit::Unlimited },
		Transact {
			origin_kind: origin_kind.map_or(xcm::v2::OriginKind::Superuser, |o| o),
			require_weight_at_most: require_weight_at_most
				.map_or(Weight::from_parts(REF_WEIGHT, PROOF_WEIGHT), |w| w),
			call: encoded_call.into(),
		},
	]))
	.encode()
}

fn build_xcm(pallet_id: PalletId, call: Vec<u8>) -> Vec<u8> {
	let encoded_call = encode_pallet_call(pallet_id, call);
	// ToDo: Need to change to Asset
	let fees = MultiAsset { id: Concrete(Here.into()), fun: Fungible(10000) };
	let xcm = transact_xcm(fees, None, None, encoded_call);

	xcm
}

pub fn try_queue_dmp<T: Config>(
	para_id: ParaId,
	pallet_id: PalletId,
	encoded_call: Vec<u8>,
) -> DispatchResult {
	let config = <configuration::Pallet<T>>::config();
	let xcm = build_xcm(pallet_id, encoded_call);
	if let Err(dmp::QueueDownwardMessageError::ExceedsMaxMessageSize) =
		<dmp::Pallet<T>>::queue_downward_message(&config, ParaId::from(para_id).into(), xcm)
	{
		log::error!(
			target: "runtime::system_token_manager",
			"sending 'dmp' failed."
		);
	};
	Ok(())
}

pub fn do_teleport_asset<T>(
	beneficiary: T::AccountId,
	amount: &T::Balance,
	asset_multi_loc: MultiLocation,
	is_relay: bool,
) where
	T: pallet_xcm::Config + pallet_assets::Config,
	u32: From<BlockNumberFor<T>>,
	<<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::AccountId: From<AccountIdOf<T>>,
	[u8; 32]: From<<T as frame_system::Config>::AccountId>,
	u128: From<<T as pallet_assets::Config>::Balance>,
{
	let dest_para_id = match asset_multi_loc.clone().interior() {
		X3(Junction::Parachain(para_id), _, _) => *para_id,
		_ => 1000,
	};
	let parents: u8 = if is_relay { 0 } else { 1 };
	let _ = pallet_xcm::Pallet::<T>::limited_teleport_assets(
		<T as frame_system::Config>::RuntimeOrigin::signed(beneficiary.clone().into()),
		Box::new(xcm::VersionedMultiLocation::V3(MultiLocation {
			parents,
			interior: X1(Junction::Parachain(dest_para_id)),
		})),
		Box::new(
			Junction::AccountId32 { network: None, id: beneficiary.clone().into() }
				.into_location()
				.into(),
		),
		Box::new(
			MultiAsset { id: Concrete(asset_multi_loc), fun: Fungible(amount.clone().into()) }
				.into(),
		),
		0,
		xcm::v3::WeightLimit::Unlimited,
	);
}
