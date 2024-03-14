
use super::{types::*, *};
use sp_std::vec;

impl<T: Config> TaaV for Pallet<T> {
	type Vote = PotVote<T::AccountId, SystemTokenAssetIdOf<T>, VoteWeightOf<T>>;
	type Weight = VoteWeightOf<T>;
	type Error = DispatchError;

	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
		// Try decode
		let vote = Self::Vote::decode(&mut &bytes[..]).map_err(|_| Error::<T>::ErrorDecode)?;
		log::info!("😇😇😇 Vote: {:?}", vote);
		let PotVote { asset, candidate, weight } = vote;
		let adjusted = T::SystemTokenInterface::adjusted_weight(&asset, weight);
		T::Voting::vote(candidate.clone(), adjusted);
		Self::deposit_event(Event::<T>::Voted { who: candidate });
		Ok(())
	}

	fn handle_vote(_vote: Self::Vote) {
		// We don't handle vote here
	}
}

impl<T: Config>
	RuntimeConfigProvider<SystemTokenBalanceOf<T>, SystemTokenWeightOf<T>> for Pallet<T>
{
	type Error = DispatchError;

	fn system_token_config() -> Result<SystemTokenConfig<SystemTokenWeightOf<T>>, Self::Error> {
		Ok(ActiveSystemConfig::<T>::get())
	}

	fn para_fee_rate() -> Result<SystemTokenWeightOf<T>, Self::Error> {
		// Relay chain's fee rate is same as base weight
		Ok(ActiveSystemConfig::<T>::get().base_weight())
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalanceOf<T>> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}
