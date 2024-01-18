use super::{
	fee::{ExtrinsicMetadata, Mode},
	token::*,
};

use sp_std::vec::Vec;

#[allow(missing_docs)]
/// API that handles Runtime configuration including System Token
pub trait InfraConfigInterface {
	/// Set base weight for the `para_id` parachain by Relay-chain governance
	fn set_base_config(para_id: SystemTokenParaId, base_system_token_detail: BaseSystemTokenDetail);
	/// Set fee table for the `para_id` parachain by Relay-chain governance
	fn set_fee_table(
		para_id: SystemTokenParaId,
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
		fee: SystemTokenBalance,
	);
	/// Set fee rate for the `para_id` parachain by Relay-chain governance
	fn set_para_fee_rate(para_id: SystemTokenParaId, fee_rate: SystemTokenWeight);
	/// Set runtime state configuration for the `para_id` parachain by Relay-chain governance
	fn set_runtime_state(para_id: SystemTokenParaId);
	/// Update weight of System Token for the `para_id` parachain by Relay-chain governance
	fn update_system_token_weight(
		para_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	);
	/// Register `original` System Token's local asset for the `para_id` parachain by Relay-chain
	/// governance
	fn register_system_token(
		para_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	);
	/// Create local asset for `Wrapped` System Token for the `para_id` parachain by Relay-chain
	/// governance
	fn create_wrapped_local(
		para_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		currency_type: Option<Fiat>,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
		asset_link_parent: u8,
		original: SystemTokenId,
	);
	fn deregister_system_token(
		para_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		is_unlink: bool,
	);
}

/// API for providing Runtime(e.g parachain) configuration which would be set by Relay-chain
/// governance
pub trait RuntimeConfigProvider {
	/// General error type
	type Error;

	/// Base system token weight of Runtime
	fn base_system_token_configuration() -> Result<BaseSystemTokenDetail, Self::Error>;
	/// Para fee rate of Runtime
	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error>;
	/// Fee for each extrinsic set in fee table
	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance>;
	/// State of Runtime
	fn runtime_state() -> Mode;
}
