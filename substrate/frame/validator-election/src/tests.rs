use frame_support::assert_ok;

use super::{
	NumberOfSeedTrustValidators as SeedTrustNum, TotalNumberOfValidators as TotalValidatorsNum, *,
};
use crate::mock::{RuntimeOrigin as TestOrigin, *};

#[test]
fn config_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(TotalValidatorsNum::<TestRuntime>::get(), 3);
		assert_eq!(SeedTrustNum::<TestRuntime>::get(), 3);
		assert_eq!(ForceEra::<TestRuntime>::get(), Forcing::NotForcing);
		assert_eq!(SeedTrustValidatorPool::<TestRuntime>::get().len(), 3);
		assert_eq!(CurrentEra::<TestRuntime>::get().unwrap(), 0);
		let current_era = CurrentEra::<TestRuntime>::get().unwrap();
		assert_eq!(StartSessionIndexPerEra::<TestRuntime>::get(current_era).unwrap(), 0);
	});
}

#[test]
fn session_and_era_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(StartSessionIndexPerEra::<TestRuntime>::get(0).unwrap(), 0);
		progress_session(1);
		let current_era = CurrentEra::<TestRuntime>::get().unwrap();
		assert_eq!(current_era, 0);
		progress_session(2);
		progress_session(3);
		progress_session(4);
		progress_session(5);
		let current_era = CurrentEra::<TestRuntime>::get().unwrap();
		assert_eq!(current_era, 1);
		assert_eq!(StartSessionIndexPerEra::<TestRuntime>::get(current_era).unwrap(), 5);
	})
}

#[test]
fn pot_works() {
	ExtBuilder::default()
		.pot_enable(true)
		.vote_status(|| create_mock_vote_status(2, true))
		.build_and_execute(|| {
			// Scenario 1
			// Gensis state
			assert_eq!(SeedTrustNum::<TestRuntime>::get(), 3);
			assert_ok!(InfraVoting::set_number_of_validators(TestOrigin::root(), 0, 0));
			assert_eq!(SeedTrustNum::<TestRuntime>::get(), 2);
			assert_eq!(PotValidatorPool::<TestRuntime>::get().counts(), 2);
			assert_eq!(
				PotValidatorPool::<TestRuntime>::get().status,
				vec![
					(sp_keyring::Sr25519Keyring::Dave.to_account_id(), 2),
					(sp_keyring::Sr25519Keyring::Ferdie.to_account_id(), 2)
				]
			);
			// Let's roll to era 1
			// We should have 2 Seed Trust and 1 Pot Validator
			for i in 1..=5 {
				progress_session(i);
			}
			let current_era = CurrentEra::<TestRuntime>::get().unwrap();
			assert_eq!(current_era, 1);
			assert_eq!(PotValidators::<TestRuntime>::get().len(), 1);
			assert_eq!(
				*infra_voting_events().last().unwrap(),
				Event::ValidatorsElected {
					validators: vec![
						sp_keyring::Sr25519Keyring::Alice.to_account_id(),
						sp_keyring::Sr25519Keyring::Bob.to_account_id(),
						sp_keyring::Sr25519Keyring::Dave.to_account_id(),
					],
					pot_enabled: true
				}
			);
			// Scenario2: Ferdie got more vote
			// Let's roll to era 2
			// We should have 2 Seed Trust and 1 Pot Validator
			let mut mock_vote_status = MockVoteStatus::create_mock_pot(2, true);
			mock_vote_status
				.increase_vote_point(sp_keyring::Sr25519Keyring::Ferdie.to_account_id());
			let voting_status: VotingStatus<TestRuntime> = mock_vote_status.into();
			PotValidatorPool::<TestRuntime>::put(voting_status);
			for i in 6..=10 {
				progress_session(i);
			}
			assert_eq!(
				*infra_voting_events().last().unwrap(),
				Event::ValidatorsElected {
					validators: vec![
						sp_keyring::Sr25519Keyring::Alice.to_account_id(),
						sp_keyring::Sr25519Keyring::Bob.to_account_id(),
						sp_keyring::Sr25519Keyring::Ferdie.to_account_id(),
					],
					pot_enabled: true
				}
			);
			progress_block(11);
			let queued_keys =
				Session::queued_keys().iter().map(|v| v.0.clone()).collect::<Vec<AccountId>>();
			assert_eq!(
				queued_keys,
				vec![
					sp_keyring::Sr25519Keyring::Alice.to_account_id(),
					sp_keyring::Sr25519Keyring::Bob.to_account_id(),
					sp_keyring::Sr25519Keyring::Ferdie.to_account_id(),
				],
			)
		})
}
