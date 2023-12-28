use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
	types::token::SystemTokenId,
};
use bounded_collections::{BoundedVec, ConstU32};
use softfloat::F64;
use sp_core::crypto::AccountId32;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

/// Account Id type of vote candidate. Should be equal to the AccountId type of the Relay Chain
pub type VoteAccountId = AccountId32;
/// Weight of vote which is weight of transaction and asset id
pub type VoteWeight = F64;
/// Which asset to vote for
pub type VoteAssetId = u32;

pub const MAX_VOTE_NUM: u32 = 16 * 1024;
use serde::{Deserialize, Serialize};

/// Aggregated votes with maximum amount `MAX_VOTE_NUM`
pub type PotVotesResult = BoundedVec<PotVote, ConstU32<MAX_VOTE_NUM>>;

/// Convert pot votes to Vec<(AccountId32, F64)>
pub fn convert_pot_votes(votes: PotVotesResult) -> Vec<(AccountId32, F64)> {
	let converted_votes: Vec<(AccountId32, F64)> = votes
		.into_iter()
		.map(|vote| {
			// Assuming AccountId32 and F64 are the types you want to use for account_id and
			// vote_weight respectively. Also assuming that VoteAccountId can be transformed into
			// AccountId32, and VoteWeight can be transformed into F64.
			let account_id = vote.account_id;
			let vote_weight = vote.vote_weight;

			(account_id, vote_weight)
		})
		.collect();

	converted_votes
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Default, Hash))]
/// Single Pot vote type
pub struct PotVote {
	#[allow(missing_docs)]
	pub system_token_id: SystemTokenId,
	#[allow(missing_docs)]
	pub account_id: VoteAccountId,
	#[allow(missing_docs)]
	pub vote_weight: VoteWeight,
}

impl PotVote {
	#[allow(missing_docs)]
	pub fn new(
		system_token_id: SystemTokenId,
		account_id: VoteAccountId,
		vote_weight: VoteWeight,
	) -> Self {
		Self { system_token_id, account_id, vote_weight }
	}
}

impl PotVote {
	/// Type cast from PotVote to PotVoteU128
	pub fn to_u128(&self) -> PotVoteU128 {
		let vote_weight = self.vote_weight.to_i128() as u128;
		PotVoteU128 {
			system_token_id: self.system_token_id.clone(),
			account_id: self.account_id.clone(),
			vote_weight,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Default, Hash))]
pub struct PotVoteU128 {
	pub system_token_id: SystemTokenId,
	pub account_id: VoteAccountId,
	pub vote_weight: u128,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// Transaction-as-a-Vote
///
/// Vote is included in transaction and send to blockchain.
/// It is collected for every block as a form of (Asset Id, Account Id, Vote Weight).
pub struct PotVotes {
	/// Status of the votes
	pub votes: BTreeMap<(SystemTokenId, VoteAccountId), VoteWeight>,
	/// Current number of the votes.
	#[codec(compact)]
	pub vote_count: u32,
	/// Maximum number of the votes.
	#[codec(compact)]
	pub max_vote_count: u32,
}

/// An interface for dealing with vote info
impl PotVotes {
	#[allow(missing_docs)]
	pub fn new(
		system_token_id: SystemTokenId,
		candidate: VoteAccountId,
		vote_weight: VoteWeight,
	) -> Self {
		let mut votes = BTreeMap::new();
		votes.insert((system_token_id, candidate), vote_weight);
		Self { votes, vote_count: 1, max_vote_count: MAX_VOTE_NUM }
	}

	/// Update vote weight for given key (asset id, account id).
	/// Before we update the votes, check if vote count is exceeded for given period.
	/// If it is not exceeded, we update the votes. Otherwise, we do nothing.
	pub fn update_vote_weight(
		&mut self,
		system_token_id: SystemTokenId,
		vote_account_id: VoteAccountId,
		vote_weight: VoteWeight,
	) {
		let mut vote_weight = vote_weight;
		let key = (system_token_id, vote_account_id);
		// Weight for asset id already existed
		if let Some(old_weight) = self.votes.get_mut(&key) {
			// Weight for asset id already existed
			vote_weight = old_weight.add(vote_weight);
		}
		// We update value if vote count is not exceeded for given period
		if self.increase_vote_count_if_not_exceeds() {
			self.votes.insert(key, vote_weight);
		}
	}

	/// Convert from `status of the votes` to `BoundedVec<_, Max>`
	pub fn votes(&self) -> PotVotesResult {
		let res = self
			.votes
			.clone()
			.into_iter()
			.map(|(k, v)| PotVote::new(k.0, k.1, v))
			.collect::<Vec<PotVote>>();
		res.try_into().expect("PotVotesResult should be bounded")
	}

	fn increase_vote_count_if_not_exceeds(&mut self) -> bool {
		let temp = self.vote_count + 1;
		if temp.le(&self.max_vote_count) {
			self.vote_count += 1;
			return true
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn new_pot_votes(
		system_token_id: SystemTokenId,
		candidate: VoteAccountId,
		vote_weight: VoteWeight,
	) -> PotVotes {
		PotVotes::new(system_token_id, candidate, vote_weight)
	}

	#[test]
	fn do_not_insert_value_if_exceeds_works() {
		let candidate: VoteAccountId = AccountId32::new([0u8; 32]);
		let system_token_id = SystemTokenId::new(2000, 50, 99);
		let vote_weight: VoteWeight = F64::from_i128(1);
		let mut pot_votes = new_pot_votes(system_token_id.clone(), candidate.clone(), vote_weight);
		for _ in 1..MAX_VOTE_NUM + 1 {
			pot_votes.update_vote_weight(system_token_id, candidate.clone(), 1);
		}
		assert_eq!(pot_votes.vote_count, MAX_VOTE_NUM);
	}

	#[test]
	fn get_votes_works() {
		let candidate: VoteAccountId = AccountId32::new([0u8; 32]);
		let vote_weight: VoteWeight = F64::from_i128(1);
		let mut pot_votes =
			new_pot_votes(SystemTokenId::new(2000, 50, 99), candidate.clone(), vote_weight);
		pot_votes.update_vote_weight(
			SystemTokenId::new(2000, 50, 98),
			candidate.clone(),
			vote_weight,
		);
		pot_votes.update_vote_weight(
			SystemTokenId::new(2000, 50, 97),
			candidate.clone(),
			vote_weight,
		);
		pot_votes.update_vote_weight(
			SystemTokenId::new(2000, 50, 96),
			candidate.clone(),
			vote_weight,
		);
		sp_std::if_std! {
			println!("Votes : {:?}", pot_votes.votes());
		}
		assert_eq!(pot_votes.vote_count, 4);
		let expected: PotVotesResult = vec![
			PotVote::new(SystemTokenId::new(2000, 50, 99), candidate.clone(), vote_weight),
			PotVote::new(SystemTokenId::new(2000, 50, 98), candidate.clone(), vote_weight),
			PotVote::new(SystemTokenId::new(2000, 50, 97), candidate.clone(), vote_weight),
			PotVote::new(SystemTokenId::new(2000, 50, 96), candidate.clone(), vote_weight),
		]
		.try_into()
		.expect("PotVotesResult should be bounded");

		assert_eq!(pot_votes.votes(), expected);
	}
}
