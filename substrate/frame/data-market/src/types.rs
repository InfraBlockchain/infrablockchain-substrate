use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type PurchaseId = u128;
pub type Quantity = u128;
pub type IssuerWeight = u32;

/// Common size is up to 100 bytes
pub const MAX_TEXT_SIZE: u32 = 1_000_000;
// TOTAL_FEE_RATIO is 100%(indicates 10_000)
pub const TOTAL_FEE_RATIO: u32 = 10_000;
// MIN_PLATFORM_FEE_RATIO is fixed at a minimum of 10%
pub const MIN_PLATFORM_FEE_RATIO: u32 = 1_000;

pub type AnyText = BoundedVec<u8, ConstU32<MAX_TEXT_SIZE>>;

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

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash, Debug))]
pub struct DataPurchaseRegisterDetails<AccountId, BlockNumber, Balance, AnyText> {
	pub data_buyer: AccountId,
	pub data_buyer_info: DataBuyerInfo<AnyText>,
	pub data_purchase_info: DataPurchaseInfo<AnyText>,
	pub data_verifiers: Vec<AccountId>,
	pub purchase_deadline: BlockNumber,
	pub system_token_asset_id: u32,
	pub quantity: Quantity,
	pub price_per_data: Balance,
	pub data_issuer_fee_ratio: u32,
	pub data_owner_fee_ratio: u32,
	pub purchase_status: PurchaseStatus,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum PurchaseStatus {
	Active,
	Finished,
	Stale,
}

pub(crate) enum TransferFrom<T: Config> {
	Origin(T::AccountId),
	Escrow,
}
