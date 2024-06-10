pub use crate::{self as pallet_newnal, *};
use frame_support::{parameter_types, traits::Everything};
use frame_system::EnsureRoot;
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

pub type MockBalance = u128;
pub type MockAccountId = AccountId32;
pub type MockBlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE_SS58: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
pub const BOB_SS58: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage} = 2,
		URAuth: pallet_newnal::{Pallet, Call, Storage, Event<T>} = 99,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = MockBlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = frame_support::traits::ConstU64<5>;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxOracleMembers: u32 = 5;
}

impl pallet_newnal::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
	type URAuthParser = URAuthParser<Self>;
	type MaxOracleMembers = MaxOracleMembers;
	type MaxURIByOracle = ConstU32<100>;
	type VerificationPeriod = ConstU64<3>;
	type MaxRequest = ConstU32<5>;
	type AuthorizedOrigin = EnsureRoot<MockAccountId>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1)); // For 'Event'
	ext
}

pub struct ExtBuilder {
	pub oracle_member_count: u32,
}

pub struct MockURAuthHelper<Account> {
	pub mock_doc_manager: MockURAuthDocManager,
	pub mock_prover: MockProver<Account>,
}

impl<Account: Encode> MockURAuthHelper<Account> {
	pub fn default(
		uri: Option<String>,
		account_id: Option<String>,
		timestamp: Option<String>,
		challenge_value: Option<String>,
	) -> Self {
		let account_id = account_id.map_or(String::from(ALICE_SS58), |id| id);
		Self {
			mock_doc_manager: MockURAuthDocManager::new(
				uri.map_or(String::from("https://www.website1.com"), |uri| uri),
				format!("{}{}", "did:infra:ua:", account_id),
				challenge_value.map_or(String::from("E40Bzg8kAvOIjswwxc29WaQCHuOKwoZC"), |cv| cv),
				timestamp.map_or(String::from("2023-07-28T10:17:21Z"), |t| t),
				None,
				None,
			),
			mock_prover: MockProver(Default::default()),
		}
	}

	pub fn deconstruct_urauth_doc(
		&self,
		uri: Option<String>,
	) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
		self.mock_doc_manager.deconstruct(uri)
	}

	pub fn create_raw_payload(&mut self, proof_type: ProofType<Account>) -> Vec<u8> {
		self.mock_prover.raw_payload(proof_type)
	}

	pub fn create_sr25519_signature(
		&mut self,
		signer: sp_keyring::AccountKeyring,
		proof_type: ProofType<Account>,
	) -> Signature {
		self.mock_prover.create_sr25519_signature(signer, proof_type)
	}

	pub fn create_signature(
		&mut self,
		signer: sp_keyring::AccountKeyring,
		proof_type: ProofType<Account>,
	) -> MultiSignature {
		self.mock_prover.create_signature(signer, proof_type)
	}

	pub fn bounded_uri(&self, of: Option<String>) -> URI {
		self.mock_doc_manager.bounded_uri(of)
	}

	pub fn owner_did(&self) -> String {
		self.mock_doc_manager.owner_did.clone()
	}

	pub fn raw_owner_did(&self) -> OwnerDID {
		self.mock_doc_manager
			.owner_did
			.as_bytes()
			.to_vec()
			.try_into()
			.expect("Too long!")
	}

	pub fn challenge_value(&self) -> Randomness {
		self.mock_doc_manager.challenge_value.as_bytes().to_vec()[..]
			.try_into()
			.unwrap()
	}

	pub fn generate_did(&self, account_id: &str) -> String {
		format!("{}{}", "did:infra:ua:", account_id)
	}

	pub fn generate_json(&mut self, proof_type: String, proof: String) -> Vec<u8> {
		self.mock_doc_manager.generate_json(proof_type, proof).as_bytes().to_vec()
	}
}

#[derive(Clone)]
pub enum ProofType<Account: Encode> {
	Request(URI, OwnerDID, MockBlockNumber),
	Challenge(URI, OwnerDID, Vec<u8>, Vec<u8>),
	Update(URI, URAuthDoc<Account>, OwnerDID, MockBlockNumber),
}

pub struct MockProver<Account>(PhantomData<Account>);

impl<Account: Encode> MockProver<Account> {
	pub fn new() -> Self {
		Self(Default::default())
	}

	fn raw_payload(&self, proof_type: ProofType<Account>) -> Vec<u8> {
		let raw = match proof_type {
			ProofType::Request(uri, owner_did, nonce) => (uri, owner_did, nonce).encode(),
			ProofType::Challenge(uri, owner_did, challenge, timestamp) =>
				(uri, owner_did, challenge, timestamp).encode(),
			ProofType::Update(uri, urauth_doc, owner_did, nonce) => {
				let URAuthDoc {
					id,
					created_at,
					updated_at,
					multi_owner_did,
					identity_info,
					content_metadata,
					copyright_info,
					access_rules,
					asset,
					data_source,
					..
				} = urauth_doc;

				(
					uri,
					id,
					created_at,
					updated_at,
					multi_owner_did,
					identity_info,
					content_metadata,
					copyright_info,
					access_rules,
					asset,
					data_source,
					owner_did,
					nonce,
				)
					.encode()
			},
		};

		if raw.len() > 256 {
			sp_io::hashing::blake2_256(&raw).to_vec()
		} else {
			raw
		}
	}

	fn create_sr25519_signature(
		&mut self,
		signer: sp_keyring::AccountKeyring,
		proof_type: ProofType<Account>,
	) -> Signature {
		let raw_payload = self.raw_payload(proof_type);
		signer.sign(&raw_payload)
	}

	fn create_signature(
		&mut self,
		signer: sp_keyring::AccountKeyring,
		proof_type: ProofType<Account>,
	) -> MultiSignature {
		let raw_payload = self.raw_payload(proof_type);
		let sig = signer.sign(&raw_payload);
		sig.into()
	}
}

pub struct MockURAuthDocManager {
	pub uri: String,
	pub owner_did: String,
	pub challenge_value: String,
	pub timestamp: String,
	pub proof_type: Option<String>,
	pub proof: Option<String>,
}

impl MockURAuthDocManager {
	pub fn new(
		uri: String,
		owner_did: String,
		challenge_value: String,
		timestamp: String,
		proof_type: Option<String>,
		proof: Option<String>,
	) -> Self {
		Self { uri, owner_did, challenge_value, timestamp, proof_type, proof }
	}

	fn bounded_uri(&self, of: Option<String>) -> URI {
		let opaque_uri = of.map_or(self.uri.as_bytes().to_vec(), |s| s.as_bytes().to_vec());
		opaque_uri.try_into().expect("Too Long")
	}

	fn deconstruct(&self, uri: Option<String>) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
		let uri = uri.map_or(self.uri.as_bytes().to_vec(), |s| s.as_bytes().to_vec());
		(
			uri,
			self.owner_did.as_bytes().to_vec(),
			self.challenge_value.as_bytes().to_vec(),
			self.timestamp.as_bytes().to_vec(),
		)
	}

	fn challenge_value(&mut self, proof_type: String, proof: String) {
		self.proof_type = Some(proof_type);
		self.proof = Some(proof);
	}

	fn generate_json(&mut self, proof_type: String, proof: String) -> String {
		use lite_json::Serialize;
		self.challenge_value(proof_type, proof);

		let mut object_elements = vec![];

		let object_key = "domain".chars().collect();
		object_elements
			.push((object_key, lite_json::JsonValue::String(self.uri.chars().collect())));

		let object_key = "adminDID".chars().collect();
		object_elements
			.push((object_key, lite_json::JsonValue::String(self.owner_did.chars().collect())));

		let object_key = "challenge".chars().collect();
		object_elements.push((
			object_key,
			lite_json::JsonValue::String(self.challenge_value.chars().collect()),
		));

		let object_key = "timestamp".chars().collect();
		object_elements
			.push((object_key, lite_json::JsonValue::String(self.timestamp.chars().collect())));

		let mut proof_object = vec![];
		let object_key = "type".chars().collect();
		proof_object.push((
			object_key,
			lite_json::JsonValue::String(
				self.proof_type.as_ref().expect("NO PROOF TYPE").chars().collect(),
			),
		));

		let object_key = "proofValue".chars().collect();
		proof_object.push((
			object_key,
			lite_json::JsonValue::String(self.proof.as_ref().expect("NO PROOF").chars().collect()),
		));

		let object_key = "proof".chars().collect();
		object_elements.push((object_key, lite_json::JsonValue::Object(proof_object)));

		let object_value = lite_json::JsonValue::Object(object_elements);

		// Convert the object to a JSON string.
		let json = object_value.format(4);
		let json_output = std::str::from_utf8(&json).unwrap();

		json_output.to_string()
	}
}

#[derive(Clone)]
pub struct RequestCall {
	origin: RuntimeOrigin,
	claim_type: ClaimType,
	uri: Vec<u8>,
	owner_did: Vec<u8>,
	challenge: Option<Randomness>,
	signer: MultiSigner,
	sig: MultiSignature,
}

impl RequestCall {
	pub fn new(
		origin: RuntimeOrigin,
		claim_type: ClaimType,
		uri: Vec<u8>,
		owner_did: Vec<u8>,
		challenge: Option<Randomness>,
		signer: MultiSigner,
		sig: MultiSignature,
	) -> Self {
		Self { origin, claim_type, uri, owner_did, challenge, signer, sig }
	}

	pub fn runtime_call(self) -> DispatchResult {
		match self.challenge {
			Some(_) => URAuth::request_register_ownership(
				self.origin,
				self.claim_type,
				self.uri,
				self.owner_did,
				self.challenge,
				self.signer,
				self.sig,
			),
			None => URAuth::claim_ownership(
				self.origin,
				self.claim_type,
				self.uri,
				self.owner_did,
				self.signer,
				self.sig,
			),
		}
	}
	pub fn set_origin(mut self, origin: RuntimeOrigin) -> Self {
		self.origin = origin;
		self
	}
	pub fn set_claim_type(mut self, claim_type: ClaimType) -> Self {
		self.claim_type = claim_type;
		self
	}
	pub fn set_uri(mut self, uri: Vec<u8>) -> Self {
		self.uri = uri;
		self
	}
	pub fn set_owner_did(mut self, owner_did: Vec<u8>) -> Self {
		self.owner_did = owner_did;
		self
	}
	pub fn set_challenge(mut self, challenge: Option<Randomness>) -> Self {
		self.challenge = challenge;
		self
	}
	pub fn set_signer(mut self, signer: MultiSigner) -> Self {
		self.signer = signer;
		self
	}
	pub fn set_sig(mut self, sig: MultiSignature) -> Self {
		self.sig = sig;
		self
	}
}

pub struct AddURIByOracleCall {
	pub origin: RuntimeOrigin,
	pub claim_type: ClaimType,
	pub uri: Vec<u8>,
}

impl AddURIByOracleCall {
	pub fn runtime_call(&self) -> DispatchResult {
		URAuth::add_uri_by_oracle(self.origin.clone(), self.claim_type.clone(), self.uri.clone())
	}
	pub fn set_claim_type(mut self, claim_type: ClaimType) -> Self {
		self.claim_type = claim_type;
		self
	}
	pub fn set_uri(mut self, uri: Vec<u8>) -> Self {
		self.uri = uri;
		self
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let mut ext = sp_io::TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
	}
}

pub fn run_to_block(n: BlockNumberFor<Test>) {
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		<URAuth as Hooks<BlockNumberFor<Test>>>::on_initialize(System::block_number());
	}
}

pub fn debug_doc<Account>(urauth_doc: &URAuthDoc<Account>)
where
	Account: Encode + sp_std::fmt::Debug,
{
	println!("URAUTH DOCUMENT => {:?}", urauth_doc);
	println!("");
	println!("DOCUMENT SIZE => {:?} bytes", urauth_doc.encode().len());
}
