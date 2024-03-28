use super::{
	fee::{ExtrinsicMetadata, Mode},
	token::Fiat,
};
use crate::*;
use codec::{Decode, Encode};
use sp_std::vec::Vec;

/// System Token configuration for transaction fee calculation
#[derive(
	Encode,
	Decode,
	Clone,
	PartialEq,
	Eq,
	Default,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct SystemConfig {
	/// Detail of base system token
	pub base_system_token_detail: BaseSystemTokenDetail,
	/// Scale of weight for calculating tx fee
	pub weight_scale: u128,
	/// Base fee rate for para_fee_rate
	pub base_para_fee_rate: u128,
}

#[derive(RuntimeDebug)]
pub enum InitError {
	/// Base system token is not initialized
	InvalidBaseSystemTokenDetail,
	/// Weight scale is not initialized
	InvalidWeightScale,
}

impl SystemConfig {
	pub fn check_validity(&self) -> Result<(), InitError> {
		if self.base_system_token_detail.base_weight == 0 {
			return Err(InitError::InvalidBaseSystemTokenDetail)
		}
		if self.weight_scale == 0 {
			return Err(InitError::InvalidWeightScale)
		}
		Ok(())
	}

	pub fn panic_if_not_validated(&self) {
		if let Err(err) = self.check_validity() {
			panic!("System configuration is not initalized: {:?}\nSCfg:\n{:#?}", err, self);
		}
	}
}
#[derive(
	Encode,
	Decode,
	Clone,
	PartialEq,
	Eq,
	Default,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	serde::Serialize,
	serde::Deserialize,
)]
/// Detail of base system token
pub struct BaseSystemTokenDetail {
	/// Currency type of base system token
	pub base_currency: Fiat,
	/// Weight of base system token
	pub base_weight: u128,
	/// Decimal of base system token
	pub base_decimals: u8,
}

impl BaseSystemTokenDetail {
	pub fn new(fiat: Fiat, base_weight: u128, decimals: u8) -> Self {
		Self { base_currency: fiat, base_weight, base_decimals: decimals }
	}
}

/// API for providing Infra-* Runtime configuration
pub trait RuntimeConfigProvider<Balance> {
	/// General error type
	type Error;

	/// System configuration
	fn system_config() -> Result<SystemConfig, Self::Error>;
	/// Para fee rate of Infra-* Runtime
	fn para_fee_rate() -> Result<Balance, Self::Error>;
	/// Query for tx fee of `ext` extrinsic
	fn fee_for(ext: ExtrinsicMetadata) -> Option<Balance>;
	/// State of Infar-* Runtime
	fn runtime_state() -> Mode;
}

/// Transaction-as-a-Vote
pub trait TaaV {
	/// Error type while processing vote
	type Error;

	/// Try to decode for given opaque `vote` and process `PotVote`
	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error>;
}
