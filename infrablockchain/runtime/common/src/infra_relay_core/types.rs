use super::*;

pub type SystemTokenAssetIdOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
pub type SystemTokenBalanceOf<T> = <<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type SystemTokenWeightOf<T> =
	<<T as Config>::Fungibles as InspectSystemToken<<T as frame_system::Config>::AccountId>>::SystemTokenWeight;
pub type VoteWeightOf<T> = <<T as Config>::Voting as PotInterface<<T as frame_system::Config>::AccountId>>::VoteWeight;

pub trait RelayChainPolicy {

	/// Destination ID type
	type DestId;
	/// Balance type of System Token
	type Balance;

	/// Update fee table for `dest_id` Runtime
	fn update_fee_table(dest_id: Self::DestId, pallet_name: Vec<u8>, call_name: Vec<u8>, fee: Self::Balance);
	/// Update fee rate for `dest_id` Runtime
	fn update_para_fee_rate(dest_id: Self::DestId, fee_rate: Self::Balance);
	/// Set runtime state for `dest_id` Runtime
	fn update_runtime_state(dest_id: Self::DestId);
}

/// API for interacting with registered System Token
// pub trait SystemTokenInterface<SystemTokenId, VoteWeight> {
// 	/// Adjust the vote weight calculating exchange rate.
// 	fn adjusted_weight(system_token_id: &SystemTokenId, vote_weight: VoteWeight) -> VoteWeight;
// }
