use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::AccountIdConversion, types::token::*, DispatchError};
use sp_std::vec;

use xcm::opaque::{
	latest::prelude::*,
	lts::{AssetId::Concrete, Fungibility::Fungible, Junction, MultiAsset, MultiLocation},
};

pub fn root_account<T: frame_system::Config>() -> T::AccountId {
	frame_support::PalletId(*b"infra/rt").into_account_truncating()
}

pub fn sovereign_account<T: frame_system::Config>() -> T::AccountId {
	frame_support::PalletId(*b"infrapid").into_account_truncating()
}

pub fn do_teleport_asset<AccountId: Clone + Encode, Sender: SendXcm>(
	beneficiary: &AccountId,
	amount: &SystemTokenBalance,
	asset_multi_loc: &MultiLocation,
	is_relay: bool,
) -> Result<(), DispatchError> {
	let parents: u8 = if is_relay { 0 } else { 1 };
	let dest_para_id = match asset_multi_loc.interior() {
		X3(Junction::Parachain(para_id), _, _) => *para_id,
		_ => 1000,
	};
	let raw_acc: [u8; 32] = beneficiary
		.clone()
		.using_encoded(|mut acc| <[u8; 32]>::decode(&mut acc))
		.map_err(|_| "Failed to encode account")?;
	let assets: MultiAssets =
		MultiAsset { id: Concrete(asset_multi_loc.clone()), fun: Fungible(amount.clone()) }.into();
	let max_assets = assets.len() as u32;
	let dest = MultiLocation { parents, interior: Junction::Parachain(dest_para_id).into() };
	let message = Xcm(vec![
		WithdrawAsset(assets),
		InitiateTeleport {
			assets: AllCounted(1).into(),
			dest,
			xcm: Xcm(vec![
				// TODO: maybe need `BuyExecution` if failed
				DepositAsset {
					assets: Wild(AllCounted(max_assets)),
					beneficiary: AccountId32 { id: raw_acc, network: None }.into(),
				},
			]),
		},
	]);
	if let Err(e) = send_xcm::<Sender>(dest, message) {
		log::info!(
			target: "runtime::system_token_helper",
			"Error sending teleport XCM. {:?}", e
		);
	};
	Ok(())
}
