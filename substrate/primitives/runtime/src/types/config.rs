
use super::{
    token::SystemTokenWeight,
    fee::{ExtrinsicMetadata, Mode},
};

/// API for providing Runtime(e.g parachain) configuration which would be set by Relay-chain governance
pub trait RuntimeConfigProvider {
    /// Type for System Token balance
    type Balance;

    /// Para fee rate of Runtime
	fn fee_rate() -> SystemTokenWeight;
    /// Fee for each extrinsic
	fn fee_for(ext: ExtrinsicMetadata) -> Option<Self::Balance>;
    /// State of Runtime
	fn runtime_state() -> Mode;
}