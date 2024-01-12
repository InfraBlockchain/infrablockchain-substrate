use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
	serde::{Deserialize, Serialize},
	RuntimeDebug, MaxEncodedLen
};
use sp_std::vec::Vec;

#[allow(missing_docs)]
#[derive(Encode, Decode, Eq, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub enum Mode {
	#[default]
	Bootstrap,
	Normal,
}

#[derive(
	Clone, Encode, Decode, Eq, PartialEq, PartialOrd, Ord, RuntimeDebug, Default, TypeInfo, Serialize,
	Deserialize,
)]
/// We used it for getting fee from fee table.
pub struct ExtrinsicMetadata {
	pallet_name: Vec<u8>,
	call_name: Vec<u8>,
}

impl ExtrinsicMetadata {
	#[allow(missing_docs)]
	pub fn new<Pallet: Encode, Call: Encode>(pallet_name: Pallet, call_name: Call) -> Self {
		Self { pallet_name: pallet_name.encode(), call_name: call_name.encode() }
	}
}
