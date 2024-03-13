use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
};
use softfloat::F64;
use sp_std::prelude::*;

pub use types::*;

pub type VoteWeight = F64;

pub mod types {

	use super::*;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Default, Hash))]
	/// Single Pot vote type
	pub struct PotVote<AccountId, AssetId, Weight = F64> {
		/// `asset` which is used for vote
		pub asset: AssetId,
		/// Subject of the vote
		pub candidate: AccountId,
		/// Absolute amount of vote
		pub weight: Weight,
	}

	impl<AccountId, AssetId, Weight> PotVote<AccountId, AssetId, Weight> {
		/// Create new instance of vote
		pub fn new(asset: AssetId, candidate: AccountId, weight: Weight) -> Self {
			Self { asset, candidate, weight }
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::crypto::AccountId32;

	#[derive(Encode, Decode)]
	pub struct MockSystemTokenId {
		para_id: Option<u32>,
		pallet_id: u8,
		asset_id: u128,
	}

	#[test]
	fn decode_works() {
		let vote = PotVote::new(
			MockSystemTokenId { para_id: Some(1), pallet_id: 1, asset_id: 1 },
			AccountId32::new([0; 32]),
			F64::from_i128(10),
		);
		let bytes = vote.encode();
		if let Ok(vote) = PotVote::<AccountId32, MockSystemTokenId, F64>::decode(&bytes) {
			println!("{:?}", vote);
		}
	}
}
