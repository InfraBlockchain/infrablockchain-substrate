use crate::{self as pallet_validator_management, *};
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, Hooks, OneSessionHandler},
};
use sp_core::{ByteArray, H256};
use sp_keyring::Sr25519Keyring::*;
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup},
	types::{VoteAccountId, VoteWeight},
	AccountId32, BuildStorage,
};
use std::collections::BTreeMap;

pub const BLOCK_TIME: u64 = 1000;

/// The AccountId alias in this test module.
pub(crate) type AccountId = AccountId32;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;
frame_support::construct_runtime!(
	pub enum TestRuntime
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		ValidatorManagement: pallet_validator_management::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub static TotalNumberOfValidators: u32 = 5;
	pub static MinVotePointsThreshold: u32 = 1;
	pub static SessionsPerEra: u32 = 5;
}

impl pallet_validator_management::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type SessionsPerEra = SessionsPerEra;
	type InfraVoteAccountId = VoteAccountId;
	type InfraVotePoints = VoteWeight;
	type NextNewSession = Session;
	type SessionInterface = Self;
	type CollectiveInterface = ();
	type RewardInterface = ();
}

parameter_types! {
	pub static ValidatorAccounts: BTreeMap<AccountId, AccountId> = BTreeMap::new();
}

pub const KEY_TYPE: sp_core::crypto::KeyTypeId = sp_application_crypto::key_types::DUMMY;

mod app {
	use sp_application_crypto::{app_crypto, key_types::DUMMY, sr25519};
	app_crypto!(sr25519, DUMMY);
}

pub type MockAuthorityId = app::Public;

/// Another session handler struct to test on_disabled.
pub struct OtherSessionHandler;
impl OneSessionHandler<AccountId> for OtherSessionHandler {
	type Key = MockAuthorityId;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_disabled(_validator_index: u32) {}
}

impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
	type Public = MockAuthorityId;
}

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub other: OtherSessionHandler,
	}
}

pub struct TestValidatorIdOf;
impl TestValidatorIdOf {
	pub fn set(v: BTreeMap<AccountId, AccountId>) {
		ValidatorAccounts::mutate(|m| *m = v);
	}
}

impl Convert<AccountId, Option<AccountId>> for TestValidatorIdOf {
	fn convert(x: AccountId) -> Option<AccountId> {
		ValidatorAccounts::get().get(&x).cloned()
	}
}

parameter_types! {
	// Only for test
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}
impl pallet_session::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type Keys = SessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = ValidatorElection;
	type SessionHandler = (OtherSessionHandler,);
	type ValidatorId = AccountId;
	type ValidatorIdOf = TestValidatorIdOf;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	total_number_of_validators: u32,
	number_of_seed_trust_validators: u32,
	seed_trust_validators: Vec<AccountId>,
	initialize_first_session: bool,
	is_pot_enable_at_genesis: bool,
	vote_status: Vec<(VoteAccountId, VoteWeight)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			total_number_of_validators: 3,
			number_of_seed_trust_validators: 2,
			seed_trust_validators: vec![],
			initialize_first_session: true,
			is_pot_enable_at_genesis: false,
			vote_status: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn total_number_of_validators(mut self, total_num: u32) -> Self {
		self.total_number_of_validators = total_num;
		self
	}

	pub fn number_of_seed_trust_validators(mut self, total_num: u32) -> Self {
		self.number_of_seed_trust_validators = total_num;
		self
	}

	pub fn seed_trust_validators(mut self, seed_trust_validators: Vec<AccountId>) -> Self {
		self.seed_trust_validators = seed_trust_validators;
		self
	}

	pub fn initialize_fist_session(mut self, init: bool) -> Self {
		self.initialize_first_session = init;
		self
	}

	pub fn pot_enable(mut self, is_enable: bool) -> Self {
		self.is_pot_enable_at_genesis = is_enable;
		self
	}

	pub fn vote_status(mut self, create_mock_vote_status: impl FnOnce() -> MockVoteStatus) -> Self {
		let vote_status = create_mock_vote_status();
		self.vote_status = vote_status.0;
		self
	}

	fn app_public(keyring: sp_keyring::Sr25519Keyring) -> app::Public {
		match keyring {
			Alice => app::Public::from_slice(&[0; 32]).unwrap(),
			Bob => app::Public::from_slice(&[1; 32]).unwrap(),
			Charlie => app::Public::from_slice(&[2; 32]).unwrap(),
			Dave => app::Public::from_slice(&[3; 32]).unwrap(),
			Eve => app::Public::from_slice(&[4; 32]).unwrap(),
			Ferdie => app::Public::from_slice(&[5; 32]).unwrap(),
			_ => app::Public::from_slice(&[6; 32]).unwrap(),
		}
	}

	fn build(self) -> sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::<TestRuntime>::default().build_storage().unwrap();

		let seed_trust_validators =
			vec![Alice.to_account_id(), Bob.to_account_id(), Charlie.to_account_id()];
		let account_keyring = vec![Alice, Bob, Charlie, Dave, Eve, Ferdie];

		let _ = pallet_validator_management::GenesisConfig::<TestRuntime> {
			seed_trust_validators: seed_trust_validators.clone(),
			total_validator_slots: self.total_number_of_validators,
			seed_trust_slots: self.number_of_seed_trust_validators,
			is_pot_enable_at_genesis: self.is_pot_enable_at_genesis,
			vote_status_at_genesis: self.vote_status,
			..Default::default()
		}
		.assimilate_storage(&mut storage);

		let _ = pallet_session::GenesisConfig::<TestRuntime> {
			keys: seed_trust_validators
				.into_iter()
				.zip(account_keyring)
				.map(|(id, keyring)| {
					(id.clone(), id.clone(), SessionKeys { other: Self::app_public(keyring) })
				})
				.collect(),
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);

		if self.initialize_first_session {
			ext.execute_with(|| {
				System::set_block_number(1);
				<Session as Hooks<BlockNumber>>::on_initialize(1);
			});
		}
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();

		ext.execute_with(test);
	}
}

/// Progress to the given block, triggering session and era changes as we progress.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn progress_block(n: BlockNumber) {
	for b in (System::block_number() + 1)..=n {
		System::set_block_number(b);
		Session::on_initialize(b);
	}
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn progress_session(session_index: SessionIndex) {
	let end: BlockNumber = (session_index as BlockNumber) * Period::get();
	progress_block(end);
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MockVoteStatus(pub Vec<(VoteAccountId, VoteWeight)>);

impl Default for MockVoteStatus {
	fn default() -> Self {
		Self(vec![])
	}
}

impl From<MockVoteStatus> for VotingStatus<TestRuntime> {
	fn from(value: MockVoteStatus) -> Self {
		Self { status: value.0 }
	}
}

impl MockVoteStatus {
	fn create_mock_account(num: usize) -> Vec<VoteAccountId> {
		let mut mock_accounts = vec![];
		let accounts = vec![Alice, Dave, Ferdie, Eve];
		for i in 0..num {
			mock_accounts.push(accounts[i].to_account_id());
		}
		mock_accounts
	}

	pub fn create_mock_pot(num: usize) -> Self {
		let mut mock_pot = vec![];
		let mock_accounts = Self::create_mock_account(num);
		mock_accounts.into_iter().for_each(|acc| {
			let vote_point = if acc == Dave.to_account_id() { 2 } else { 3 };
			mock_pot.push((acc, vote_point as VoteWeight));
		});
		Self(mock_pot)
	}

	pub fn increase_vote_point(&mut self, who: VoteAccountId) {
		self.0.iter_mut().for_each(|vote_status| {
			if vote_status.0 == who {
				vote_status.1 += 1;
			}
		})
	}
}
/// There will be three candidates for testing
/// Dave, Eve, Ferdie
pub(crate) fn create_mock_vote_status(num: usize) -> MockVoteStatus {
	MockVoteStatus::create_mock_pot(num)
}

pub(crate) fn validator_management_events() -> Vec<crate::Event<TestRuntime>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(
			|e| if let RuntimeEvent::ValidatorElection(inner) = e { Some(inner) } else { None },
		)
		.collect()
}
