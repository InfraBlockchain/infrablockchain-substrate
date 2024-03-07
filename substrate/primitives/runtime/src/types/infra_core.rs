use super::{
	fee::{ExtrinsicMetadata, Mode},
	token::{Fiat, SystemTokenConfig},
};

use codec::Encode;
use sp_std::vec::Vec;

/// API that updates Infra-* Runtime configuration
// TODO: Remove 'ParaId', 'SystemTokenId'
pub trait UpdateInfraConfig<Location> {

	/// `AssetId` for InfraBlockchain(e.g MultiLocation) 
	type AssetId: Encode + Into<Location>;
	/// 'ParaId' for XCM destination
	type ParaId: Encode;
	/// Associated `Weight` type for InfraBlockchain
	type SystemTokenWeight: Encode;
	/// Associated `Balance` type for InfraBlockchain
	type Balance: Encode;

	/// Update fee table for `dest_id` Runtime
	fn update_fee_table(
		asset_id: Self::AssetId,
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
		fee: Self::Balance,
	);
	/// Update fee rate for `dest_id` Runtime
	fn update_para_fee_rate(dest_id: Self::ParaId, fee_rate: Self::SystemTokenWeight);
	/// Set runtime state for `dest_id` Runtime
	fn update_runtime_state(dest_id: Self::ParaId);
	/// Update `SystemTokenWeight` for `dest_id` Runtime
	fn update_system_token_weight(
		asset_id: Self::AssetId,
		system_token_weight: Self::SystemTokenWeight,
	);
	/// Register `Original` System Token for `dest_id` Runtime(e.g `set_sufficient=true`)
	fn register_system_token(
		dest_id: Self::ParaId,
		asset_id: Self::AssetId,
		system_token_weight: Self::SystemTokenWeight,
	);
	/// Create local asset of `Wrapped` System Token for `dest_id` Runtime
	fn create_wrapped_local(
		dest_id: Self::ParaId,
		original: Location,
		currency_type: Fiat,
		min_balance: Self::Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: Self::SystemTokenWeight,
		asset_link_parent: u8,
	);
	/// Deregister `Original/Wrapped` System Token for `dest_id` Runtime
	fn deregister_system_token(
		dest_id: Self::ParaId,
		asset_id: Location,
		is_unlink: bool,
	);
}

/// API for providing Infra-* Runtime configuration
pub trait RuntimeConfigProvider<SystemTokenBalance, SystemTokenWeight> {
	/// General error type
	type Error;

	/// System Token configuration
	fn system_token_config() -> Result<SystemTokenConfig<SystemTokenWeight>, Self::Error>;
	/// Para fee rate of Infra-* Runtime
	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error>;
	/// Query for tx fee of `ext` extrinsic
	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance>;
	/// State of Infar-* Runtime
	fn runtime_state() -> Mode;
}
