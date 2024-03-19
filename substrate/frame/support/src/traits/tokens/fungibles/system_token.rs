use softfloat::F64;
use sp_runtime::{types::{Fiat, RemoteAssetMetadata}, DispatchError};
use sp_std::vec::Vec;

use crate::traits::tokens::Balance;

use super::metadata;

/// Interface for inspecting System Token
pub trait Inspect<AccountId>: super::Inspect<AccountId> {
	/// Associate type of weight for System Token
	type SystemTokenWeight: Balance + From<F64>;

	/// Returns true if the asset is a system token which refers to `is_sufficient = true`
	fn is_system_token(asset: &Self::AssetId) -> bool;
	/// Return System Token balance of `who`. If asset is `None`, most balance of `who` will be
	/// returned
	fn balance(
		who: &AccountId,
		maybe_asset: Option<Self::AssetId>,
	) -> Option<(Self::AssetId, Self::Balance)>;
	/// Return `Self::SystemTokenWeight` of System Token
	fn system_token_weight(
		asset: Self::AssetId,
	) -> Result<Self::SystemTokenWeight, sp_runtime::DispatchError>;
	/// Inspect currency type of System Token
	fn fiat(asset: Self::AssetId) -> Result<Fiat, sp_runtime::DispatchError>;
}

/// Interface for managing System Token
pub trait Manage<AccountId>: super::InspectSystemToken<AccountId> {
	/// Register as System Token
	fn register(asset: Self::AssetId, system_token_weight: Self::SystemTokenWeight) -> Result<(), DispatchError>;
	/// Deregister as System Token
	fn deregister(asset: Self::AssetId) -> Result<(), DispatchError>;
	/// Update weight of System Token based on exchange rate
	fn update_system_token_weight(asset: Self::AssetId, system_token_weight: Self::SystemTokenWeight) -> Result<(), DispatchError>;
	/// Request register System Token
	fn request_register(asset: Self::AssetId) -> Result<(), DispatchError>;
	/// Create `Wrapped` asset
	fn touch(
		owner: AccountId,
		asset: Self::AssetId,
		currency_type: Fiat,
		min_balance: Self::Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: Self::SystemTokenWeight,
	) -> Result<(), DispatchError>;
}

/// Interface for enumerating System Token
pub trait Enumerate<AccountId>: super::InspectEnumerable<AccountId> {
	/// Returns all registered system tokens
	fn system_token_ids() -> impl IntoIterator<Item = Self::AssetId>;
	/// Returns all System Tokens of 'who'
	fn system_token_account_balances(
		who: &AccountId
	) -> impl IntoIterator<Item = (Self::AssetId, Self::Balance)>;
}

/// Interface for inspecting System Token Metadata
pub trait Metadata<AccountId>: metadata::Inspect<AccountId> {
	
	fn inner(asset: Self::AssetId) -> Result<(Fiat, Self::Balance), DispatchError>;
	
	fn system_token_metadata(asset: Self::AssetId) -> Result<RemoteAssetMetadata<Self::AssetId, Self::Balance>, DispatchError> {
		let name = Self::name(asset.clone());
		let symbol = Self::symbol(asset.clone());
		let decimals = Self::decimals(asset.clone());
		let (currency_type, min_balance) = Self::inner(asset.clone())?;
		Ok(RemoteAssetMetadata {
			asset_id: asset,
			name,
			symbol,
			currency_type,
			decimals,
			min_balance
		})
	}
}
