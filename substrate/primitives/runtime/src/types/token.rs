//! Module that defines System Token type

use crate::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
	types::vote::*,
	RuntimeDebug,
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// ParaId of Relay Chain
pub const RELAY_CHAIN_PARA_ID: SystemTokenParaId = 0;

/// General para id type for System Token
pub type SystemTokenParaId = u32;
/// General pallet id type for System Token
pub type SystemTokenPalletId = u8;
/// General asset id type for System Token
pub type SystemTokenAssetId = u32;
/// Generale weight type for System Token
pub type SystemTokenWeight = u128;
/// General balance type for System Token
pub type SystemTokenBalance = u128;

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
	pub para_id: SystemTokenParaId,
	/// PalletId on the parachain where to use the system token
	#[codec(compact)]
	pub pallet_id: SystemTokenPalletId,
	/// AssetId on the parachain where to use the system token
	#[codec(compact)]
	pub asset_id: SystemTokenAssetId,
}

impl SystemTokenId {
	/// Create new instance of `SystemTokenId`
	pub fn new(para_id: u32, pallet_id: u8, asset_id: SystemTokenAssetId) -> Self {
		Self { para_id, pallet_id, asset_id }
	}

	/// Clone `self` and return new instance of `SystemTokenId`
	pub fn asset_id(&self) -> SystemTokenAssetId {
		self.clone().asset_id
	}
}

/// API for interacting with local assets on Runtime
pub trait LocalAssetProvider<Asset, Account> {
	/// Get a list of local assets created on local chain
	fn system_token_list() -> Vec<Asset>;
	/// Get the most account balance of given `asset_id`
	fn get_most_account_system_token_balance(
		asset_ids: impl IntoIterator<Item = Asset>,
		account: Account,
	) -> Asset;
}

/// API to create/destory local assets related to System Token
pub trait LocalAssetManager {
	type Error;
	/// Create local asset with metadata which refers to `wrapped` System Token
	fn create_wrapped_local(
		asset_id: SystemTokenAssetId,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
	/// Promote local asset to System Token when registered(e.g `is_sufficient` to `true`)
	fn promote(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
	/// Demote System Token to local asset(e.g `is_sufficient` to `false`)
	fn demote(asset_id: SystemTokenAssetId) -> Result<(), Self::Error>;
	/// Update weight of System Token(e.g Exhange rate has been changed)
	fn update_system_token_weight(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
}

/// API for interacting with registered System Token
pub trait SystemTokenInterface {
	/// Check the system token is registered.
	fn is_system_token(system_token: &SystemTokenId) -> bool;
	/// Convert para system token to original system token.
	fn convert_to_original_system_token(wrapped_token: &SystemTokenId) -> Option<SystemTokenId>;
	/// Adjust the vote weight calculating exchange rate.
	fn adjusted_weight(system_token: &SystemTokenId, vote_weight: VoteWeight) -> VoteWeight;
}

impl SystemTokenInterface for () {
	fn is_system_token(_system_token: &SystemTokenId) -> bool {
		false
	}
	fn convert_to_original_system_token(_wrapped_token: &SystemTokenId) -> Option<SystemTokenId> {
		None
	}
	fn adjusted_weight(_system_token: &SystemTokenId, _vote_weight: VoteWeight) -> VoteWeight {
		Default::default()
	}
}

pub trait AssetLinkInterface<AssetId> {
	type Error;

	fn link(asset_id: &AssetId, parents: u8, original: SystemTokenId) -> Result<(), Self::Error>;
	fn unlink(asset_id: &AssetId) -> Result<(), Self::Error>;
}

impl<AssetId> AssetLinkInterface<AssetId> for () {
	type Error = ();

	fn link(
		_asset_id: &AssetId,
		_parents: u8,
		_original: SystemTokenId,
	) -> Result<(), Self::Error> {
		Ok(())
	}
	fn unlink(_asset_id: &AssetId) -> Result<(), Self::Error> {
		Ok(())
	}
}
