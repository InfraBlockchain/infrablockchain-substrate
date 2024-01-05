use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Common size is up to 100 bytes
pub const MAX_TEXT_SIZE: u32 = 1_000_000;

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
