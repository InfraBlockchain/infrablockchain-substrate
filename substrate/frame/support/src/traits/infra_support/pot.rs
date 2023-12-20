use sp_runtime::types::{SystemTokenId, VoteAccountId, VoteWeight};
/// An interface for dealing with vote info
pub trait VotingHandler {
	fn update_pot_vote(who: VoteAccountId, system_token_id: SystemTokenId, vote_weight: VoteWeight);
}

impl VotingHandler for () {
	fn update_pot_vote(
		_who: VoteAccountId,
		_system_token_id: SystemTokenId,
		_vote_weight: VoteWeight,
	) {
	}
}
