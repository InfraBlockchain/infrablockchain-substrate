//! Types for InfraBlockchain

mod fee;
pub mod token;
mod vote;

pub use self::{
	fee::ExtrinsicMetadata,
	token::{
		AssetId, PalletId, ParaId, SystemTokenId, SystemTokenLocalAssetProvider,
		SystemTokenWeight, RELAY_CHAIN_PARA_ID,
	},
	vote::{PotVote, PotVotes, PotVotesResult, VoteAccountId, VoteAssetId, VoteWeight},
};
