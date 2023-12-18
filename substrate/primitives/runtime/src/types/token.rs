//! Module that defines System Token type

use crate::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo, RuntimeDebug
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// ParaId of Relay Chain
pub const RELAY_CHAIN_PARA_ID: ParaId = 0;

/// Identifier of parachain
pub type ParaId = u32;
/// Identifier of pallet
pub type PalletId = u8;
/// Identifier of asset
pub type AssetId = u32;
/// Weight of system token.
///
/// For example,
pub type SystemTokenWeight = u128;
/// Data structure for Original system tokens
#[derive(
	Clone,
	Encode,
	Decode,
	Copy,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	RuntimeDebug,
	Default,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Hash, Serialize, Deserialize))]
pub struct SystemTokenId {
	/// ParaId where to use the system token. Especially, we assigned the relaychain as ParaID = 0
	#[codec(compact)]
	pub para_id: ParaId,
	/// PalletId on the parachain where to use the system token
	#[codec(compact)]
	pub pallet_id: PalletId,
	/// AssetId on the parachain where to use the system token
	#[codec(compact)]
	pub asset_id: AssetId,
}

impl SystemTokenId {
	/// Create new instance of `SystemTokenId`
	pub fn new(para_id: u32, pallet_id: u8, asset_id: AssetId) -> Self {
		Self { para_id, pallet_id, asset_id }
	}
}

#[allow(missing_docs)]
#[derive(Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub enum RuntimeState {
	#[default]
	Bootstrap,
	Normal
}

#[allow(missing_docs)]
/// API for local asset
pub trait SystemTokenLocalAssetProvider<Asset, Account> {

	fn runtime_state() -> RuntimeState;
	/// Get a list of local assets created on local chain
	fn system_token_list() -> Option<Vec<Asset>>;
	/// Get the most account balance
	fn get_most_account_system_token_balance(
		asset_ids: impl IntoIterator<Item = Asset>,
		account: Account
	) -> Asset;
}
