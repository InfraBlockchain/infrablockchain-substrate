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
		Voted { candidate: VoteAccountId, vote_system_token: SystemTokenId, weight: VoteWeight },
	}

	#[pallet::error]
	pub enum Error<T> {
		NotSystemToken,
	}
}

impl<T: Config> VotingHandler for Pallet<T> {
	fn update_pot_vote(who: VoteAccountId, system_token_id: SystemTokenId, vote_weight: VoteWeight) {
		Self::do_update_pot_vote(system_token_id, who, vote_weight);
	}
}

impl<T: Config> Pallet<T> {
	/// Update vote weight for given (asset_id, candidate)
	fn do_update_pot_vote(
		vote_system_token: SystemTokenId,
		candidate: VoteAccountId,
		vote_weight: VoteWeight,
	) {
		// ToDo: Should check whether this is system token or not
		let weight =
			T::SystemTokenManager::adjusted_weight(&vote_system_token, vote_weight);
		if vote_system_token.is_boot() {
			return
		}
		T::VotingHandler::update_vote_status(candidate.clone(), weight);
		Self::deposit_event(Event::<T>::Voted {
			candidate,
			vote_system_token,
			weight,
		})
	}
}
