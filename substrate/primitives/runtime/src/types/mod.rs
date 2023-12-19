//! Types for InfraBlockchain

mod fee;
pub mod token;
mod vote;

pub use self::{
	fee::ExtrinsicMetadata,
	token::{
		AssetId, PalletId, ParaId, SystemTokenId, SystemTokenLocalAssetProvider, SystemTokenWeight,
	},
	vote::{
		convert_pot_votes, PotVote, PotVotes, PotVotesResult, PotVotesU128Result, VoteAccountId,
		VoteAssetId, VoteWeight,
	},
};
