use frame_support::traits::ibs_support::pot::VotingHandler;
pub use pallet::*;
use pallet_validator_election::VotingInterface;
use runtime_parachains::system_token_manager::SystemTokenInterface;
use sp_runtime::types::{SystemTokenId, VoteAccountId, VoteWeight};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Updating vote type
		type VotingHandler: VotingInterface<Self>;
		/// Managing System Token
		type SystemTokenManager: SystemTokenInterface;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted { candidate: VoteAccountId, asset_id: SystemTokenId, weight: VoteWeight },
	}

	#[pallet::error]
	pub enum Error<T> {
		NotSystemToken,
	}
}

impl<T: Config> VotingHandler for Pallet<T> {
	fn update_pot_vote(who: VoteAccountId, asset_id: SystemTokenId, vote_weight: VoteWeight) {
		Self::do_update_pot_vote(asset_id, who, vote_weight);
	}
}

impl<T: Config> Pallet<T> {
	/// Update vote weight for given (asset_id, candidate)
	fn do_update_pot_vote(
		vote_asset_id: SystemTokenId,
		vote_account_id: VoteAccountId,
		vote_weight: VoteWeight,
	) {
		// ToDo: Should check whether this is system token or not
		let adjusted_weight =
			T::SystemTokenManager::adjusted_weight(vote_asset_id.clone(), vote_weight);
		T::VotingHandler::update_vote_status(vote_account_id.clone(), adjusted_weight);
		Self::deposit_event(Event::<T>::Voted {
			candidate: vote_account_id,
			asset_id: vote_asset_id.clone(),
			weight: adjusted_weight,
		})
	}
}
