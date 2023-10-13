use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
};
use sp_std::vec::Vec;

#[derive(
	Clone,
	Encode,
	Decode,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	sp_core::RuntimeDebug,
	Default,
	TypeInfo,
)]
/// We used it for getting fee from fee table.
pub struct ExtrinsicMetadata {
	pallet_name: Vec<u8>,
	call_name: Vec<u8>
}

impl ExtrinsicMetadata {
	pub fn new<Pallet: Encode, Call: Encode>(pallet_name: Pallet, call_name: Call) -> Self {
		Self {
			pallet_name: pallet_name.encode(),
			call_name: call_name.encode()
		}
	}
}
