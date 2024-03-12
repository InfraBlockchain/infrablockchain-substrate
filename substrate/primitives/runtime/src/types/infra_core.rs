use super::{
	fee::{ExtrinsicMetadata, Mode},
	token::{Fiat, SystemTokenConfig},
};

use codec::FullCodec;
use softfloat::F64;
use sp_std::vec::Vec;

/// API that updates Infra-* Runtime configuration
// TODO: Remove 'ParaId', 'SystemTokenId'
pub trait UpdateInfraConfig<Location, OriginId, Weight, Balance> {
	/// Update fee table for `dest_id` Runtime
	fn update_fee_table(dest_id: OriginId, pallet_name: Vec<u8>, call_name: Vec<u8>, fee: Balance);
	/// Update fee rate for `dest_id` Runtime
	fn update_para_fee_rate(dest_id: OriginId, fee_rate: Balance);
	/// Set runtime state for `dest_id` Runtime
	fn update_runtime_state(dest_id: OriginId);
	/// Register `Original` System Token for `dest_id` Runtime(e.g `set_sufficient=true`)
	fn register_system_token(dest_id: OriginId, asset_id: Location, system_token_weight: Weight);
	/// Deregister `Original/Wrapped` System Token for `dest_id` Runtime
	fn deregister_system_token(dest_id: OriginId, asset_id: Location);
	/// Create local asset of `Wrapped` System Token for `dest_id` Runtime
	fn create_wrapped(
		dest_id: OriginId,
		original: Location,
		currency_type: Fiat,
		min_balance: Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: Weight,
		asset_link_parent: u8,
	);
}

/// API for providing Infra-* Runtime configuration
pub trait RuntimeConfigProvider<Balance, Weight> {
	/// General error type
	type Error;

	/// System Token configuration
	fn system_token_config() -> Result<SystemTokenConfig<Weight>, Self::Error>;
	/// Para fee rate of Infra-* Runtime
	fn para_fee_rate() -> Result<Weight, Self::Error>;
	/// Query for tx fee of `ext` extrinsic
	fn fee_for(ext: ExtrinsicMetadata) -> Option<Balance>;
	/// State of Infar-* Runtime
	fn runtime_state() -> Mode;
}

/// Transaction-as-a-Vote
pub trait TaaV {
	/// Type of `candidate` of vote
	type AccountId;
	/// Type of `weight` of vote
	type VoteWeight: Into<F64>;
	/// Error type while processing vote
	type Error;

	/// Try to decode for given `vote` and process `PotVote`
	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error>;
}
