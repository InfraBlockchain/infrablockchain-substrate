use sp_runtime::types::{ParaId, SystemTokenId, VoteWeight};
type WrappedSystemTokenId = SystemTokenId;

/// System tokens API.
pub trait SystemTokenInterface {
	/// Check whether it is bootstrap system token.
	fn is_boot(id: ParaId) -> bool;
	/// Check the system token is registered.
	fn is_system_token(system_token: &SystemTokenId) -> bool;
	/// Convert para system token to original system token.
	fn convert_to_original_system_token(
		wrapped_token: &WrappedSystemTokenId,
	) -> Option<SystemTokenId>;
	/// Adjust the vote weight calculating exchange rate.
	fn adjusted_weight(system_token: &SystemTokenId, vote_weight: VoteWeight) -> VoteWeight;
}

impl SystemTokenInterface for () {
	fn is_boot(_id: ParaId) -> bool {
		false
	}

	fn is_system_token(_system_token: &SystemTokenId) -> bool {
		false
	}
	fn convert_to_original_system_token(
		_wrapped_token: &WrappedSystemTokenId,
	) -> Option<SystemTokenId> {
		None
	}
	fn adjusted_weight(_system_token: &SystemTokenId, _vote_weight: VoteWeight) -> VoteWeight {
		Default::default()
	}
}
