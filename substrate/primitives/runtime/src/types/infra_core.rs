
use super::{
    token::*,
    fee::{ExtrinsicMetadata, Mode},
};

use sp_std::vec::Vec;

#[allow(missing_docs)]
/// API that handles Runtime configuration including System Token
pub trait InfraConfigInterface {
    fn set_base_weight(para_id: SystemTokenParaId);
	fn set_fee_table(para_id: SystemTokenParaId, pallet_name: Vec<u8>, call_name: Vec<u8>, fee: SystemTokenBalance);
	fn set_fee_rate(para_id: SystemTokenParaId, fee_rate: SystemTokenWeight);
	fn set_runtime_state(para_id: SystemTokenParaId);
	fn set_system_token_weight(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight);
	fn register_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight);
	fn create_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight);
	fn deregister_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId);
}

/// API for providing Runtime(e.g parachain) configuration which would be set by Relay-chain governance
pub trait RuntimeConfigProvider {
    /// Error type for API
    type Error;

    /// Base system token weight of Runtime
    fn base_weight() -> Result<SystemTokenWeight, Self::Error>;
    /// Para fee rate of Runtime
	fn fee_rate() -> Result<SystemTokenWeight, Self::Error>;
    /// Fee for each extrinsic
	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance>;
    /// State of Runtime
	fn runtime_state() -> Mode;
}