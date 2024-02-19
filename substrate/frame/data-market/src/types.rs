use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type ContractId = u128;
pub type Quantity = u128;
pub type IssuerWeight = u32;

/// Common size is up to 100 bytes
pub const MAX_TEXT_SIZE: u32 = 1_000_000;
pub const MAX_ENTITIES: u32 = 2;

pub type AssetBalanceOf<T> =
	<<T as Config>::Assets as Inspect<<T as SystemConfig>::AccountId>>::Balance;
pub type AssetIdOf<T> = <<T as Config>::Assets as Inspect<<T as SystemConfig>::AccountId>>::AssetId;

pub type AnyText = BoundedVec<u8, ConstU32<MAX_TEXT_SIZE>>;

pub type ContractSigner<T> =
	BoundedBTreeMap<<T as SystemConfig>::AccountId, SignStatus, ConstU32<MAX_ENTITIES>>;

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct DataBuyerInfo<BoundedString> {
	data_buyer: BoundedString,
	description: BoundedString,
}

impl<BoundedString> DataBuyerInfo<BoundedString> {
	pub fn new(data_buyer: BoundedString, description: BoundedString) -> Self {
		Self { data_buyer, description }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct DataPurchaseInfo<BoundedString> {
	target_scope: BoundedString,
	data_scope: BoundedString,
}

impl<BoundedString> DataPurchaseInfo<BoundedString> {
	pub fn new(target_scope: BoundedString, data_scope: BoundedString) -> Self {
		Self { target_scope, data_scope }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct VerificationProof<BoundedString> {
	hash: BoundedString,
}

impl<BoundedString> VerificationProof<BoundedString> {
	pub fn new(hash: BoundedString) -> Self {
		Self { hash }
	}
}

pub(crate) enum TransferFrom<T: Config> {
	Origin(T::AccountId),
	Escrow,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash, Debug))]
pub struct DataDelegateContractDetail<AccountId, BlockNumber> {
	pub data_owner: AccountId,
	pub data_owner_info: AnyText,
	pub agency: AccountId,
	pub agency_info: AnyText,
	pub data_owner_minimum_fee_ratio: u32,
	pub deligated_data: AnyText,
	pub effective_at: BlockNumber,
	pub expired_at: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash, Debug))]
pub struct DataDelegateContractParams<AccountId, BlockNumber> {
	pub data_owner: AccountId,
	pub data_owner_info: AnyText,
	pub data_owner_minimum_fee_ratio: u32,
	pub deligated_data: AnyText,
	pub duration: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash, Debug))]
pub struct DataPurchaseContractDetail<AccountId, BlockNumber, Balance> {
	pub data_buyer: AccountId,
	pub data_buyer_info: AnyText,
	pub data_verifier: Option<AccountId>,
	pub effective_at: BlockNumber,
	pub expired_at: BlockNumber,
	pub data_purchase_info: DataPurchaseInfo<AnyText>,
	pub system_token_id: u32,
	pub agency: Option<AccountId>,
	pub agency_info: Option<AnyText>,
	pub price_per_data: Balance,
	pub deposit: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash, Debug))]
pub struct DataPurchaseContractParams<AccountId, BlockNumber, Balance> {
	pub data_buyer_info: AnyText,
	pub data_verifier: Option<AccountId>,
	pub data_purchase_info: DataPurchaseInfo<AnyText>,
	pub system_token_id: u32,
	pub agency: Option<AccountId>,
	pub price_per_data: Balance,
	pub deposit: Balance,
	pub duration: BlockNumber,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum ContractType {
	Delegate,
	Purchase,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum SignStatus {
	Unsigned,
	Signed,
	WantToTerminate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
pub struct MarketConfiguration {
	pub total_fee_ratio: u32,
	pub min_platform_fee_ratio: u32,
}
