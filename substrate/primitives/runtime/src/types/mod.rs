//! Types for InfraBlockchain

mod fee;
pub mod token;
mod vote;
mod config;

pub use self::{
	fee::{ExtrinsicMetadata, Mode},
	token::{
		AssetId as SystemTokenAssetId, PalletId as SystemTokenPalletId, ParaId as SystemTokenParaId, 
		SystemTokenId, SystemTokenLocalAssetProvider, SystemTokenWeight, SystemTokenBalance, 
		RELAY_CHAIN_PARA_ID,
	},
	vote::{
		convert_pot_votes, PotVote, PotVotes, PotVotesResult, 
		VoteAccountId, VoteAssetId, VoteWeight,
	},
	config::RuntimeConfigProvider,
};
