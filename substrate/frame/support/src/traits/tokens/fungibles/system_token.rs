
use sp_std::vec::Vec;

/// Interface for inspecting System Token 
pub trait Inspect<AccountId>: super::Inspect<AccountId> {

    /// Returns true if the asset is a system token which refers to `is_sufficient = true`
    fn is_system_token(asset: Self::AssetId) -> bool;
    /// Return System Token balance of `who`. If asset is `None`, most balance of `who` will be returned
    fn balance(who: AccountId, maybe_asset: Option<Self::AssetId>) -> Self::Balance;
}

/// Interface for managing System Token
pub trait Manage<AccountId, Metadata, SystemTokenWeight, Fiat>: super::InspectSystemTokenMetadata<AccountId, Metadata, SystemTokenWeight, Fiat> {
    /// Register as System Token 
    fn register(asset: Self::AssetId);
    /// Deregister as System Token
    fn deregister(asset: Self::AssetId);
    /// Update weight of System Token based on exchange rate
    fn update_weight(asset: Self::AssetId, weight: SystemTokenWeight);
    /// Request registering for System Token
    fn request(asset: Self::AssetId) -> Metadata;
    /// Create `Wrapped` asset
    fn touch(
        asset: Self::AssetId, 
        currency_type: Fiat,
		min_balance: Self::Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight
    );
} 

/// Interface for enumerating System Token
pub trait Enumerate<AccountId>: super::InspectEnumerable<AccountId> {
    /// Returns all system tokens
    fn system_token_ids() -> impl IntoIterator<Item = Self::AssetId>;
}

/// Interface for inspecting System Token Metadata
pub trait Metadata<AccountId, SystemTokenWeight, Fiat, RemoteAssetMetadata>: super::Inspect<AccountId> {
    
    /// Associate type of weight for System Token
    // type SystemTokenWeight;
    /// Associate type of fiat for System Token
    // type Fiat;

    /// Returns the metadata of the system token
    fn system_token_metadata(asset: Self::AssetId) -> Result<RemoteAssetMetadata, sp_runtime::DispatchError>;
    /// Return `Self::SystemTokenWeight` of System Token
    fn system_token_weight(asset: Self::AssetId) -> SystemTokenWeight;
    /// Return `Self::Fiat` of System Token
    fn fiat(asset: Self::AssetId) -> Fiat;
}

