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
	pub struct PotVote<Account, AssetId, Amount> {
		/// Subject of the vote
		pub candidate: Account,
		/// Asset used in vote
		pub asset_id: AssetId,
		/// Absolute amount of vote based on tx-fee
		pub amount: Amount,
	}

	impl<Account, AssetId, Amount> PotVote<Account, AssetId, Amount> {
		/// Create new instance of vote
		pub fn new(candidate: Account, asset_id: AssetId, amount: Amount) -> Self {
			Self { candidate, asset_id, amount }
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
			AccountId32::new([0; 32]),
			MockSystemTokenId { para_id: Some(1), pallet_id: 1, asset_id: 1 },
			10i128,
		);
		let bytes = vote.encode();
		if let Ok(vote) = PotVote::<AccountId32, MockSystemTokenId, i128>::decode(&bytes) {
			println!("{:?}", vote);
		}
	}
}
