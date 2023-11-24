use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
use core::fmt::Debug;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::traits::*;

#[derive(
	Encode, Decode, Copy, scale_info_derive::TypeInfo, Clone, PartialEq, Eq, Debug, MaxEncodedLen,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(omit_prefix)]
pub enum CurveType {
	/// BLS12-381
	Bls12381,
}

/// Defines associated types used by `did-core`.
pub trait Types: Clone + Eq {
	type BlockNumber: Member
		+ MaybeSerializeDeserialize
		+ MaybeFromStr
		+ Debug
		+ sp_std::hash::Hash
		+ Copy
		+ MaybeDisplay
		+ AtLeast32BitUnsigned
		+ Default
		+ TypeInfo
		+ MaxEncodedLen
		+ FullCodec;

	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;
}

impl<T: frame_system::Config> Types for T {
	type BlockNumber = BlockNumberFor<T>;
	type AccountId = T::AccountId;
}
