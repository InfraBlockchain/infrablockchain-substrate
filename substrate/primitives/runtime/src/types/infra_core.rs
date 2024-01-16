
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
	fn set_para_fee_rate(para_id: SystemTokenParaId, fee_rate: SystemTokenWeight);
	fn set_runtime_state(para_id: SystemTokenParaId);
	fn update_system_token_weight(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, system_token_weight: SystemTokenWeight);
	fn register_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, system_token_weight: SystemTokenWeight);
	/// Create local asset for `Wrapped` System Token
    fn create_wrapped_local(
        para_id: SystemTokenParaId, 
        asset_id: SystemTokenAssetId, 
        min_balance: SystemTokenBalance, 
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
        system_token_weight: SystemTokenWeight, 
        asset_link_parent: u8,
        original: SystemTokenId
    );
	fn deregister_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, is_unlink: bool);
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