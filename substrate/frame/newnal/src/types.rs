use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
pub use size::*;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, if_std};

pub type Rules = Vec<Rule>;
pub type DocId = [u8; 16];
pub type Randomness = [u8; 32];
pub type DIDWeight = u16;
pub type ApprovalCount = u32;
pub type Threshold = u32;
pub type URAuthDocCount = u128;
pub type URAuthChallengeValue = (Vec<u8>, Vec<u8>, Vec<u8>, URI, OwnerDID, Vec<u8>);

pub type URIFor<T> = <<T as Config>::URAuthParser as Parser<T>>::URI;
pub type URIPartFor<T> = <<T as Config>::URAuthParser as Parser<T>>::Part;
pub type ClaimTypeFor<T> = <<T as Config>::URAuthParser as Parser<T>>::ClaimType;
pub type ChallengeValueFor<T> = <<T as Config>::URAuthParser as Parser<T>>::ChallengeValue;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ClaimType {
	Domain,
	Contents { data_source: Option<Vec<u8>>, name: Vec<u8>, description: Vec<u8> },
}

impl MaxEncodedLen for ClaimType {
	fn max_encoded_len() -> usize {
		URI::max_encoded_len() + URI::max_encoded_len()
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct DidDetails<T: Config> {
	pub nonce: BlockNumberFor<T>,
}

impl<T: Config> DidDetails<T>
where
	BlockNumberFor<T>: From<u8>,
{
	pub fn default() -> Self {
		Self { nonce: Default::default() }
	}

	pub fn nonce(&self) -> BlockNumberFor<T> {
		self.nonce
	}

	pub fn try_increase_nonce(&mut self) -> DispatchResult {
		self.nonce = self.nonce.checked_add(&1u8.into()).ok_or(Error::<T>::Overflow)?;
		Ok(())
	}
}

#[derive(Encode, Decode, Clone, Eq, RuntimeDebug, TypeInfo)]
pub struct URIPart {
	pub scheme: Vec<u8>,
	pub sub_domain: Option<Vec<u8>>,
	pub host: Option<Vec<u8>>,
	pub path: Option<Vec<u8>>,
}

impl PartialEq for URIPart {
	fn eq(&self, other: &Self) -> bool {
		let any = b'*';
		if !other.scheme.contains(&any) {
			if self.scheme != other.scheme {
				return false
			}
		}
		match (self.sub_domain.clone(), other.sub_domain.clone()) {
			(Some(s), Some(o_s)) => {
				if s.len() < o_s.len() {
					return false
				}
				if let Some(i) = o_s.iter().position(|s| *s == any) {
					if i + 1 > o_s.len() {
						return false
					}
					if s[i + 1..o_s.len()] != o_s[i + 1..o_s.len()] {
						return false
					}
				} else {
					if s != o_s {
						return false
					}
				}
			},
			(None, None) => {},
			_ => return false,
		}
		match (self.host.clone(), other.host.clone()) {
			(Some(s), Some(o_s)) =>
				if s != o_s {
					return false
				},
			(None, None) => {},
			_ => return false,
		}
		match (self.path.clone(), other.path.clone()) {
			(Some(s), Some(o_s)) => {
				if s.len() < o_s.len() {
					return false
				}
				if let Some(i) = o_s.iter().position(|s| *s == any) {
					if s[0..i] != o_s[0..i] {
						return false
					}
				} else {
					if s != o_s {
						return false
					}
				}
			},
			(None, None) => {},
			_ => return false,
		}
		true
	}
}

impl URIPart {
	pub fn new(
		scheme: Vec<u8>,
		sub_domain: Option<Vec<u8>>,
		host: Option<Vec<u8>>,
		path: Option<Vec<u8>>,
	) -> Self {
		let mut default_scheme = scheme;
		if default_scheme == "http".as_bytes().to_vec() {
			default_scheme = "https".as_bytes().to_vec();
		}
		default_scheme.append(&mut "://".as_bytes().to_vec());
		Self { scheme: default_scheme, sub_domain, host, path }
	}

	pub fn full_uri(&self) -> (Option<Vec<u8>>, Vec<u8>) {
		let mut full = Vec::new();
		let mut scheme = self.scheme.clone();
		let mut maybe_scheme: Option<Vec<u8>> = None;
		if scheme != "https://".as_bytes().to_vec() && scheme != "https://".as_bytes().to_vec() {
			full.append(&mut scheme);
			maybe_scheme = Some(scheme);
		}
		let mut host = self.host.clone().map_or("".as_bytes().to_vec(), |v| v);
		let mut sub_domain = self.sub_domain.clone().map_or("".as_bytes().to_vec(), |v| v);
		let mut path = self.path.clone().map_or("".as_bytes().to_vec(), |v| v);
		if sub_domain != "www.".as_bytes().to_vec() {
			full.append(&mut sub_domain);
		}
		full.append(&mut host);
		full.append(&mut path);
		(maybe_scheme, full)
	}

	pub fn root(&self) -> Option<Vec<u8>> {
		let mut root: Vec<u8> = Vec::new();
		let mut scheme = self.scheme.clone();
		if let Some(mut host) = self.host.clone() {
			if scheme != "http://".as_bytes().to_vec() && scheme != "https://".as_bytes().to_vec() {
				root.append(&mut scheme);
			}
			root.append(&mut host);
			Some(root)
		} else {
			None
		}
	}

	pub fn is_root(&self, claim_type: &ClaimType) -> bool {
		let mut is_root: bool = false;
		// Root of 'File' and 'Dataset' has CID
		// e.g newnal://file/{cid}
		if matches!(claim_type, ClaimType::Contents { .. }) {
			let mut count = 0;
			let slash = b'/';
			if let Some(path) = self.path.clone() {
				for b in path {
					if b == slash {
						count += 1;
						if count == 2 {
							break
						}
					}
				}
			}
			if count == 1 {
				is_root = true;
			}
			return is_root
		}
		if self.sub_domain == Some("www.".as_bytes().to_vec()) && self.path == None {
			is_root = true;
		}
		is_root
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for URIPart {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let host = self.host.clone().map_or(Vec::new(), |s| s);
		let sub_domain = self.sub_domain.clone().map_or(Vec::new(), |s| s);
		let path = self.path.clone().map_or(Vec::new(), |s| s);
		write!(
			f,
			"Scheme => {:?}, Sub => {:?}, Host => {:?}, Path => {:?}",
			std::str::from_utf8(&self.scheme).unwrap(),
			std::str::from_utf8(&sub_domain).unwrap(),
			std::str::from_utf8(&host).unwrap(),
			std::str::from_utf8(&path).unwrap(),
		)
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo, MaxEncodedLen)]
pub struct DataSetMetadata<BoundedString> {
	name: BoundedString,
	description: BoundedString,
}

impl<BoundedString> DataSetMetadata<BoundedString> {
	pub fn new(name: BoundedString, description: BoundedString) -> Self {
		Self { name, description }
	}
}

/// Metadata for verifying challenge value
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct RequestMetadata {
	pub owner_did: OwnerDID,
	pub challenge_value: Randomness,
	pub claim_type: ClaimType,
	pub maybe_register_uri: URI,
}

impl RequestMetadata {
	pub fn new(
		owner_did: OwnerDID,
		challenge_value: Randomness,
		claim_type: ClaimType,
		maybe_register_uri: URI,
	) -> Self {
		Self { owner_did, challenge_value, claim_type, maybe_register_uri }
	}
}

/// Submission detail for verifying challenge value
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VerificationSubmission<T: Config> {
	pub voters: Vec<T::AccountId>,
	pub status: BTreeMap<H256, ApprovalCount>,
	pub threshold: Threshold,
}

impl<T: Config> Default for VerificationSubmission<T> {
	fn default() -> Self {
		Self { voters: Default::default(), status: BTreeMap::new(), threshold: 1 }
	}
}

impl<T: Config> VerificationSubmission<T> {
	/// Submit its verification info. Threshold will be changed based on _oracle members_.
	///
	/// ## Logistics
	/// 1. Update the threshold based on number of _oracle member_.
	/// 2. Check whether given `T::AccountId` has already submitted.
	/// 3. Check whether to end its verification. `self.check_is_end`
	///
	/// ## Errors
	/// `AlreadySubmitted`
	pub fn submit(
		&mut self,
		member_count: usize,
		submission: (T::AccountId, H256),
	) -> Result<VerificationSubmissionResult, DispatchError> {
		self.update_threshold(member_count);
		for acc in self.voters.iter() {
			if &submission.0 == acc {
				return Err(Error::<T>::AlreadySubmitted.into())
			}
		}
		self.voters.push(submission.0);
		Ok(self.check_is_end(member_count, submission.1))
	}

	/// Check whether to finish its verification and return `VerificationSubmissionResult`.
	fn check_is_end(&mut self, member_count: usize, digest: H256) -> VerificationSubmissionResult {
		let mut is_end = false;
		self.status
			.entry(digest)
			.and_modify(|v| {
				*v = v.saturating_add(1);
				if *v >= self.threshold {
					is_end = true;
				}
			})
			.or_insert(1);

		if is_end {
			return VerificationSubmissionResult::Complete
		}

		if self.threshold == 1 {
			VerificationSubmissionResult::Complete
		} else if self.voters.len() == member_count {
			VerificationSubmissionResult::Tie
		} else {
			VerificationSubmissionResult::InProgress
		}
	}

	/// Update the threshold of `VerificationSubmission` based on member count.
	/// `Threshold = (member_count * 3 / 5) + remainder`
	pub fn update_threshold(&mut self, member_count: usize) {
		let threshold = (member_count * 3 / 5) as Threshold;
		let check_sum: Threshold = if (member_count * 3) % 5 == 0 { 0 } else { 1 };
		self.threshold = threshold.saturating_add(check_sum);
	}
}

/// Result state of verifying challenge value
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum VerificationSubmissionResult {
	/// Threshold has not yet reached.
	InProgress,
	/// Number of approval of a challenge value has reached to threshold.
	Complete,
	/// Number of voters and oracle member are same.
	Tie,
}

/// A payload factory for creating message for verifying its signature.
#[derive(Decode, Clone, PartialEq, Eq)]
pub enum URAuthSignedPayload<Account, BlockNumber> {
	Request { uri: URI, owner_did: OwnerDID, nonce: BlockNumber },
	Challenge { uri: URI, owner_did: OwnerDID, challenge: Vec<u8>, timestamp: Vec<u8> },
	Update { uri: URI, newnal_doc: URAuthDoc<Account>, owner_did: OwnerDID, nonce: BlockNumber },
}

impl<Account: Encode, BlockNumber: Encode> Encode for URAuthSignedPayload<Account, BlockNumber> {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		let raw_payload = match self {
			URAuthSignedPayload::Request { uri, owner_did, nonce } =>
				(uri, owner_did, nonce).encode(),
			URAuthSignedPayload::Challenge { uri, owner_did, challenge, timestamp } =>
				(uri, owner_did, challenge, timestamp).encode(),
			URAuthSignedPayload::Update { uri, newnal_doc, owner_did, nonce } => {
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
				} = newnal_doc;

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
		if raw_payload.len() > 256 {
			f(&sp_io::hashing::blake2_256(&raw_payload)[..])
		} else {
			f(&raw_payload)
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum AccountIdSource {
	DID(Vec<u8>),
	AccountId32(AccountId32),
}

/// DID with its weight
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct WeightedDID<Account> {
	pub did: Account,
	pub weight: DIDWeight,
}

impl<Account> WeightedDID<Account> {
	pub fn new(acc: Account, weight: DIDWeight) -> Self {
		Self { did: acc, weight }
	}
}

/// Owners of `URAuthDoc`. Its entities can update the doc based on its weight and threshold.
#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo)]
pub struct MultiDID<Account> {
	pub dids: Vec<WeightedDID<Account>>,
	// Sum(weight) >= threshold
	pub threshold: DIDWeight,
}

impl<Account> MaxEncodedLen for MultiDID<Account>
where
	Account: Encode + MaxEncodedLen,
{
	fn max_encoded_len() -> usize {
		WeightedDID::<Account>::max_encoded_len() * MAX_OWNER_DID_SIZE as usize
	}
}

impl<Account: PartialEq> MultiDID<Account> {
	pub fn new(acc: Account, weight: DIDWeight) -> Self {
		Self { dids: sp_std::vec![WeightedDID::<Account>::new(acc, weight)], threshold: weight }
	}

	/// Check whether given account is owner of `URAuthDoc`
	pub fn is_owner(&self, who: &Account) -> bool {
		for weighted_did in self.dids.iter() {
			if &weighted_did.did == who {
				return true
			}
		}
		false
	}

	pub fn get_threshold(&self) -> DIDWeight {
		self.threshold
	}

	pub fn add_owner(&mut self, weighted_did: WeightedDID<Account>) {
		self.dids.push(weighted_did);
	}

	pub fn get_did_weight(self, who: &Account) -> Option<DIDWeight> {
		if let Some(weighted_did) =
			self.dids.into_iter().find(|weighted_did| &weighted_did.did == who)
		{
			return Some(weighted_did.weight)
		}
		None
	}

	/// Get sum of owners' weight
	pub fn total_weight(&self) -> DIDWeight {
		let mut total = 0;
		for did in self.dids.iter() {
			total += did.weight;
		}
		total
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum IdentityInfo {
	IdentityInfoV1 { vc: VerifiableCredential },
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StorageProvider {
	IPFS,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ContentAddress {
	storage_provider: StorageProvider,
	cid: URI,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ContentMetadata {
	MetadataV1 { content_address: ContentAddress },
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CopyrightInfo {
	Text(AnyText),
	CopyrightInfoV1 { content_address: ContentAddress },
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum AccessRule {
	AccessRuleV1 { path: AnyText, rules: Rules },
}

impl MaxEncodedLen for AccessRule {
	fn max_encoded_len() -> usize {
		AnyText::max_encoded_len() + Rule::max_encoded_len() * MAX_RULES_NUM
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Rule {
	pub user_agents: Vec<UserAgent>,
	pub allow: Vec<(ContentType, Price)>,
	pub disallow: Vec<ContentType>,
}

impl MaxEncodedLen for Rule {
	fn max_encoded_len() -> usize {
		UserAgent::max_encoded_len() * MAX_USER_AGENTS_NUM +
			(ContentType::max_encoded_len() + Price::max_encoded_len()) * 4
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum PriceUnit {
	USDPerMb,
	KRWPerMb,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Price {
	pub price: u64,
	pub decimals: u8,
	pub unit: PriceUnit,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ContentType {
	#[default]
	All,
	Image,
	Video,
	Text,
	Code,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Proof {
	ProofV1 { did: OwnerDID, proof: MultiSignature },
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, TypeInfo)]
pub struct URAuthDoc<Account> {
	pub id: DocId,
	pub created_at: u128,
	pub updated_at: u128,
	pub multi_owner_did: MultiDID<Account>,
	pub identity_info: Option<Vec<IdentityInfo>>,
	pub content_metadata: Option<ContentMetadata>,
	pub copyright_info: Option<CopyrightInfo>,
	pub access_rules: Option<Vec<AccessRule>>,
	pub asset: Option<MultiAsset>,
	pub data_source: Option<URI>,
	pub proofs: Option<Vec<Proof>>,
}

impl<Account> MaxEncodedLen for URAuthDoc<Account>
where
	Account: Encode + MaxEncodedLen,
{
	fn max_encoded_len() -> usize {
		DocId::max_encoded_len() +
			u128::max_encoded_len() +
			u128::max_encoded_len() +
			MultiDID::<Account>::max_encoded_len() +
			IdentityInfo::max_encoded_len() * MAX_MULTI_OWNERS_NUM +
			AccessRule::max_encoded_len() * MAX_ACCESS_RULES +
			CopyrightInfo::max_encoded_len() +
			ContentMetadata::max_encoded_len() +
			Proof::max_encoded_len() * MAX_MULTI_OWNERS_NUM +
			MultiAsset::max_encoded_len() +
			URI::max_encoded_len()
	}
}

impl<Account> URAuthDoc<Account>
where
	Account: PartialEq + Clone,
{
	pub fn new(
		id: DocId,
		multi_owner_did: MultiDID<Account>,
		created_at: u128,
		asset: Option<MultiAsset>,
		data_source: Option<URI>,
	) -> Self {
		Self {
			id,
			multi_owner_did,
			created_at,
			updated_at: created_at,
			identity_info: None,
			content_metadata: None,
			copyright_info: None,
			access_rules: None,
			proofs: None,
			asset,
			data_source,
		}
	}

	pub fn is_owner(&self, who: &Account) -> bool {
		self.multi_owner_did.is_owner(who)
	}

	pub fn get_threshold(&self) -> DIDWeight {
		self.multi_owner_did.threshold
	}

	pub fn get_multi_did(&self) -> MultiDID<Account> {
		self.multi_owner_did.clone()
	}

	pub fn handle_proofs(&mut self, proofs: Option<Vec<Proof>>) {
		self.remove_all_prev_proofs();
		self.proofs = proofs;
	}

	pub fn add_proof(&mut self, proof: Proof) {
		let mut some_proofs = self.proofs.take().map_or(Default::default(), |proofs| proofs);
		some_proofs.push(proof);
		self.proofs = Some(some_proofs);
	}

	pub fn update_doc(
		&mut self,
		update_doc_field: UpdateDocField<Account>,
		updated_at: u128,
	) -> Result<(), URAuthDocUpdateError> {
		self.updated_at = updated_at;
		match update_doc_field {
			UpdateDocField::MultiDID(weighted_did) => {
				self.multi_owner_did.add_owner(weighted_did);
			},
			UpdateDocField::Threshold(new) => {
				let total_weight = self.multi_owner_did.total_weight();
				if total_weight < new {
					return Err(URAuthDocUpdateError::ThresholdError)
				}
				self.multi_owner_did.threshold = new;
			},
			UpdateDocField::IdentityInfo(identity_info) => {
				self.identity_info = identity_info;
			},
			UpdateDocField::ContentMetadata(content_metadata) => {
				self.content_metadata = content_metadata;
			},
			UpdateDocField::CopyrightInfo(copyright_info) => {
				self.copyright_info = copyright_info;
			},
			UpdateDocField::AccessRules(access_rules) => {
				self.access_rules = access_rules;
			},
		};

		Ok(())
	}

	fn remove_all_prev_proofs(&mut self) {
		self.proofs = Some(Vec::new());
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdateDocField<Account> {
	MultiDID(WeightedDID<Account>),
	Threshold(DIDWeight),
	IdentityInfo(Option<Vec<IdentityInfo>>),
	ContentMetadata(Option<ContentMetadata>),
	CopyrightInfo(Option<CopyrightInfo>),
	AccessRules(Option<Vec<AccessRule>>),
}

impl<Account> MaxEncodedLen for UpdateDocField<Account>
where
	Account: Encode,
{
	fn max_encoded_len() -> usize {
		AccessRule::max_encoded_len() * MAX_ACCESS_RULES
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdateStatus<Account> {
	/// Hold updated field and its proofs. Proofs will be stored on `URAuthDoc`
	InProgress {
		field: UpdateDocField<Account>,
		proofs: Option<Vec<Proof>>,
	},
	Available,
}

impl<Account> MaxEncodedLen for UpdateStatus<Account>
where
	Account: Encode,
{
	fn max_encoded_len() -> usize {
		UpdateDocField::<Account>::max_encoded_len() +
			Proof::max_encoded_len() * MAX_MULTI_OWNERS_NUM
	}
}

/// Status for updating `URAuthDoc`
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UpdateDocStatus<Account> {
	/// Threshold for updating
	pub remaining_threshold: DIDWeight,
	pub status: UpdateStatus<Account>,
}

impl<Account> Default for UpdateDocStatus<Account> {
	fn default() -> Self {
		Self { remaining_threshold: Default::default(), status: UpdateStatus::Available }
	}
}

impl<Account: Clone> UpdateDocStatus<Account> {
	pub fn is_update_available(&self) -> bool {
		matches!(self.status, UpdateStatus::Available)
	}

	/// Handle on `UpdateStatus::Available`.
	///
	/// 1. Set its _remaining_threshold_ to threshold of `URAuthDoc`
	/// 2. Set its `UpdateStatus` to `UpdateStatus::InProgress`.
	/// Define its variant to `"to be updated"` field and _proofs_ to be `None`
	pub fn handle_available(
		&mut self,
		threshold: DIDWeight,
		update_doc_field: UpdateDocField<Account>,
	) {
		self.remaining_threshold = threshold;
		self.status = UpdateStatus::InProgress { field: update_doc_field, proofs: None };
	}

	/// Handle on `UpdateStatus::InProgress`
	///
	/// 1. Add proof
	/// 2. Decrease its threshold with amount of _did_weight_
	///
	/// ## Error
	/// `ProofMissing`
	pub fn handle_in_progress(
		&mut self,
		did_weight: DIDWeight,
		update_doc_field: UpdateDocField<Account>,
		proof: Proof,
	) -> Result<(), UpdateDocStatusError> {
		if let Some(proofs) = self.add_proof(proof) {
			self.calc_remaining_threshold(did_weight);
			self.status =
				UpdateStatus::InProgress { field: update_doc_field, proofs: Some(proofs) };
		} else {
			return Err(UpdateDocStatusError::ProofMissing.into())
		}

		Ok(())
	}

	/// Get all proofs of `UpdateStatus::InProgress { proofs, ..}`. Otherwise, `None`
	pub fn get_proofs(&self) -> Option<Vec<Proof>> {
		match self.status.clone() {
			UpdateStatus::InProgress { proofs, .. } => proofs,
			_ => None,
		}
	}

	/// Add given proof on `UpdateStatus::InProgress { .. }`. Otherwise, `None`
	fn add_proof(&mut self, proof: Proof) -> Option<Vec<Proof>> {
		let maybe_proofs = match self.status.clone() {
			UpdateStatus::InProgress { proofs, .. } => {
				let mut ps = if let Some(proofs) = proofs { proofs } else { Default::default() };
				ps.push(proof);
				Some(ps)
			},
			_ => None,
		};
		maybe_proofs
	}

	/// Decrease threshold with amount to _did_weight.
	fn calc_remaining_threshold(&mut self, did_weight: DIDWeight) {
		self.remaining_threshold = self.remaining_threshold.saturating_sub(did_weight);
	}
}

/// Errors that may happen on update `URAuthDoc`
#[derive(PartialEq, RuntimeDebug)]
pub enum URAuthDocUpdateError {
	/// Threshold should be less than total weight of owners
	ThresholdError,
}

impl sp_runtime::traits::Printable for URAuthDocUpdateError {
	fn print(&self) {
		"URAuthDocUpdateError".print();
		match self {
			Self::ThresholdError => "GreaterThanTotalWeight".print(),
		}
	}
}

/// Errors that may happen on `UpdateDocStatus`
#[derive(PartialEq, RuntimeDebug)]
pub enum UpdateDocStatusError {
	/// Proof should be existed on update`URAuthDoc`
	ProofMissing,
}

impl sp_runtime::traits::Printable for UpdateDocStatusError {
	fn print(&self) {
		"UpdateDocStatusError".print();
		match self {
			Self::ProofMissing => "ProofMissingOnUpdate".print(),
		}
	}
}

pub trait Parser<T: Config> {
	type URI;
	type Part: Clone + sp_std::fmt::Debug + PartialEq;
	type ClaimType;
	type ChallengeValue: Default;

	fn parse_uri(raw_uri: &Vec<u8>, claim_type: &ClaimType) -> Result<Self::Part, DispatchError>;

	fn parse_parent_uris(
		raw_uri: &Vec<u8>,
		claim_type: &ClaimType,
	) -> Result<Vec<URI>, DispatchError>;

	fn parse_challenge_json(
		challenge_json: &Vec<u8>,
	) -> Result<Self::ChallengeValue, DispatchError>;
}

pub struct URAuthParser<T>(PhantomData<T>);
impl<T: Config> URAuthParser<T> {
	pub fn try_parse(raw_uri: &Vec<u8>, claim_type: &ClaimType) -> Result<URIPart, DispatchError> {
		if raw_uri.len() < 3 {
			return Err(Error::<T>::BadURI.into())
		}
		let full_uri =
			sp_std::str::from_utf8(&raw_uri).map_err(|_| Error::<T>::ErrorConvertToString)?;
		let uri = Url::parse(full_uri).map_err(|_| {
			if_std! { println!("Error parsing {:?}", full_uri )}
			Error::<T>::GeneralURINotSupportedYet
		})?;
		let uri_part = uri.convert(claim_type).map_err(|_| Error::<T>::ErrorOnParse)?;
		Ok(uri_part)
	}

	/// Parse the given uri and will return the list of the parsed `URI`
	fn try_parse_parent_uris(
		raw_uri: &Vec<u8>,
		claim_type: &ClaimType,
	) -> Result<Vec<URI>, DispatchError> {
		let uri_part = Self::try_parse(raw_uri, claim_type)?;
		let mut is_root = false;
		if uri_part.is_root(claim_type) {
			is_root = true;
		}
		// Only parse if there is root. Otherwise, return `Err`
		if let Some(base) = uri_part.root() {
			let base =
				sp_std::str::from_utf8(&base).map_err(|_| Error::<T>::ErrorConvertToString)?;
			let (maybe_protocol, full_uri) = uri_part.full_uri();
			let uri =
				sp_std::str::from_utf8(&full_uri).map_err(|_| Error::<T>::ErrorConvertToString)?;
			if is_root {
				let bounded_uri: URI =
					uri.as_bytes().to_vec().try_into().map_err(|_| Error::<T>::OverMaxSize)?;
				return Ok(sp_std::vec![bounded_uri])
			}
			let mut uris: Vec<URI> = Vec::new();
			let mut parent_uri = uri;
			// 1. Parse path
			while let Some(i) = parent_uri.rfind('/') {
				if parent_uri == base {
					break
				}
				parent_uri = &uri[0..i];
				sp_std::if_std! { println!("{:?}", parent_uri) }
				let bounded_uri: URI = parent_uri
					.as_bytes()
					.to_vec()
					.try_into()
					.map_err(|_| Error::<T>::OverMaxSize)?;
				uris.push(bounded_uri);
			}
			// 2. Parse sub-domain
			while let Some(i) = parent_uri.find('.') {
				if parent_uri == base {
					break
				}
				parent_uri = &parent_uri[i + 1..];
				let mut parent_uri_bytes = parent_uri.as_bytes().to_vec();
				if let Some(mut protocol) = maybe_protocol.clone() {
					protocol.append(&mut parent_uri_bytes);
					parent_uri_bytes = protocol;
				}
				sp_std::if_std! { println!("{:?}", sp_std::str::from_utf8(&parent_uri_bytes).expect("")) }
				let bounded_uri: URI =
					parent_uri_bytes.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
				uris.push(bounded_uri);
			}
			Ok(uris)
		} else {
			Err(Error::<T>::ErrorOnParse.into())
		}
	}

	/// Method for finding _json_value_ based on `field_name` and `sub_field`
	///
	/// ## Error
	/// `BadChallengeValue`
	fn find_json_value(
		json_object: &lite_json::JsonObject,
		field_name: &str,
		sub_field: Option<&str>,
	) -> Result<Option<Vec<u8>>, DispatchError> {
		let sub = sub_field.map_or("", |s| s);
		let (_, json_value) = json_object
			.iter()
			.find(|(field, _)| field.iter().copied().eq(field_name.chars()))
			.ok_or(Error::<T>::BadChallengeValue)?;
		match json_value {
			lite_json::JsonValue::String(v) =>
				Ok(Some(v.iter().map(|c| *c as u8).collect::<Vec<u8>>())),
			lite_json::JsonValue::Object(v) => Self::find_json_value(v, sub, None),
			_ => Ok(None),
		}
	}
}
impl<T: Config> Parser<T> for URAuthParser<T> {
	type URI = URI;
	type Part = URIPart;
	type ClaimType = ClaimType;
	type ChallengeValue = (Vec<u8>, Vec<u8>, Vec<u8>, URI, OwnerDID, Vec<u8>);

	fn parse_uri(
		raw_uri: &Vec<u8>,
		claim_type: &Self::ClaimType,
	) -> Result<Self::Part, DispatchError> {
		Self::try_parse(raw_uri, claim_type)
	}

	fn parse_parent_uris(
		raw_uri: &Vec<u8>,
		claim_type: &Self::ClaimType,
	) -> Result<Vec<Self::URI>, DispatchError> {
		Self::try_parse_parent_uris(raw_uri, claim_type)
	}

	fn parse_challenge_json(
		challenge_json: &Vec<u8>,
	) -> Result<Self::ChallengeValue, DispatchError> {
		let json_str =
			sp_std::str::from_utf8(challenge_json).map_err(|_| Error::<T>::ErrorConvertToString)?;

		return match lite_json::parse_json(json_str) {
			Ok(obj) => match obj {
				// ToDo: Check domain, admin_did, challenge
				lite_json::JsonValue::Object(obj) => {
					let uri = Self::find_json_value(&obj, "domain", None)?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let owner_did = Self::find_json_value(&obj, "adminDID", None)?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let challenge = Self::find_json_value(&obj, "challenge", None)?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let timestamp = Self::find_json_value(&obj, "timestamp", None)?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let proof_type = Self::find_json_value(&obj, "proof", Some("type"))?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let hex_proof = Self::find_json_value(&obj, "proof", Some("proofValue"))?
						.ok_or(Error::<T>::BadChallengeValue)?;
					let mut proof = [0u8; 64];
					hex::decode_to_slice(hex_proof, &mut proof as &mut [u8])
						.map_err(|_| Error::<T>::ErrorDecodeHex)?;
					let mut raw_payload: Vec<u8> = Default::default();
					let bounded_uri: URI = uri.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
					let bounded_owner_did: OwnerDID =
						owner_did.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
					URAuthSignedPayload::<T::AccountId, BlockNumberFor<T>>::Challenge {
						uri: bounded_uri.clone(),
						owner_did: bounded_owner_did.clone(),
						challenge: challenge.clone(),
						timestamp,
					}
					.using_encoded(|m| raw_payload = m.to_vec());

					Ok((
						proof.to_vec(),
						proof_type,
						raw_payload,
						bounded_uri,
						bounded_owner_did,
						challenge,
					))
				},
				_ => Err(Error::<T>::BadChallengeValue.into()),
			},
			Err(_) => Err(Error::<T>::BadChallengeValue.into()),
		}
	}
}

pub mod size {
	use super::*;

	/// Maximum number of `URAuthDoc` owners we expect in a single `MultiDID` value. Note this is
	/// not (yet) enforced, and just serves to provide a sensible `max_encoded_len` for `MultiDID`.
	pub const MAX_MULTI_OWNERS_NUM: usize = 5;

	/// Maximum number of `access_rules` we expect in a single `MultiDID` value. Note this is not
	/// (yet) enforced, and just serves to provide a sensible `max_encoded_len` for `MultiDID`.
	pub const MAX_ACCESS_RULES: usize = 100;

	/// Maximum number of `user agents` we expect in a single `MultiDID` value. Note this is not
	/// (yet) enforced, and just serves to provide a sensible `max_encoded_len` for `MultiDID`.
	pub const MAX_USER_AGENTS_NUM: usize = 5;

	/// Maximum number of `rule` we expect in a single `MultiDID` value. Note this is not (yet)
	/// enforced, and just serves to provide a sensible `max_encoded_len` for `MultiDID`.
	pub const MAX_RULES_NUM: usize = 20;

	/// URI is up to 3 KB
	pub const MAX_URI_SIZE: u32 = 3 * 1024;

	pub type URI = BoundedVec<u8, ConstU32<MAX_URI_SIZE>>;

	/// Owner did is up to 64 bytes
	pub const MAX_OWNER_DID_SIZE: u32 = 64;

	pub type OwnerDID = BoundedVec<u8, ConstU32<MAX_OWNER_DID_SIZE>>;

	/// Common size is up to 100 bytes
	pub const MAX_COMMON_SIZE: u32 = 100;

	pub type AnyText = BoundedVec<u8, ConstU32<MAX_COMMON_SIZE>>;

	pub type UserAgent = AnyText;

	/// Encoded size of VC is up to 1 KB.
	pub const MAX_IDENTITY_INFO: u32 = 1024;

	pub type VerifiableCredential = BoundedVec<u8, ConstU32<MAX_IDENTITY_INFO>>;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;

	#[test]
	fn max_encoded_len() {
		println!("{:?}", IdentityInfo::max_encoded_len());
		println!("{:?}", Rule::max_encoded_len());
		println!("{:?}", AccessRule::max_encoded_len());
		println!(
			"MAX URAUTH DOCUMENT SIZE is {:?} MB",
			URAuthDoc::<AccountId32>::max_encoded_len() as f32 / 1_000_000f32
		);
	}

	#[test]
	fn verification_submission_update_status_works() {
		use sp_keyring::AccountKeyring::*;

		// Complete
		let mut s1: VerificationSubmission<Test> = Default::default();
		let h1 = BlakeTwo256::hash(&1u32.to_le_bytes());
		s1.submit(3, (Alice.to_account_id(), h1)).unwrap();
		let res = s1.submit(3, (Alice.to_account_id(), h1));
		assert_eq!(
			res,
			Err(sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
				index: 99,
				error: [24, 0, 0, 0],
				message: Some("AlreadySubmitted")
			}))
		);
		println!("{:?}", s1);

		// Tie
		let mut s2: VerificationSubmission<Test> = Default::default();
		let h1 = BlakeTwo256::hash(&1u32.to_le_bytes());
		let h2 = BlakeTwo256::hash(&2u32.to_le_bytes());
		let h3 = BlakeTwo256::hash(&3u32.to_le_bytes());
		let res = s2.submit(3, (Alice.to_account_id(), h1)).unwrap();
		assert_eq!(res, VerificationSubmissionResult::InProgress);
		let res = s2.submit(3, (Bob.to_account_id(), h2)).unwrap();
		assert_eq!(res, VerificationSubmissionResult::InProgress);
		let res = s2.submit(3, (Charlie.to_account_id(), h3)).unwrap();
		assert_eq!(res, VerificationSubmissionResult::Tie);

		// 1 member and submit
		let mut s3: VerificationSubmission<Test> = Default::default();
		let h1 = BlakeTwo256::hash(&1u32.to_le_bytes());
		let res = s3.submit(1, (Alice.to_account_id(), h1)).unwrap();
		assert_eq!(res, VerificationSubmissionResult::Complete);
	}

	#[test]
	fn verification_submission_dynamic_threshold_works() {
		let mut submission: VerificationSubmission<Test> = Default::default();
		submission.update_threshold(1);
		assert_eq!(submission.threshold, 1);
		submission.update_threshold(2);
		assert_eq!(submission.threshold, 2);
		submission.update_threshold(3);
		assert_eq!(submission.threshold, 2);
		submission.update_threshold(4);
		assert_eq!(submission.threshold, 3);
		submission.update_threshold(5);
		assert_eq!(submission.threshold, 3);
	}

	// cargo t -p pallet-newnal --lib -- types::tests::deconstruct_works --exact --nocapture
	#[test]
	fn parse_works() {
		// URI with length less than minimum should fail
		assert!(
			URAuthParser::<Test>::try_parse(&"in".as_bytes().to_vec(), &ClaimType::Domain).is_err()
		);

		// Full URI with domain
		let raw_uri = "https://sub2.sub1.instagram.com/user1/feed".as_bytes().to_vec();
		let uri_part = URAuthParser::<Test>::try_parse(&raw_uri, &ClaimType::Domain).unwrap();
		assert_eq!(uri_part.scheme, "https://".as_bytes().to_vec());
		assert_eq!(uri_part.host, Some("instagram.com".as_bytes().to_vec()));
		assert_eq!(uri_part.sub_domain, Some("sub2.sub1.".as_bytes().to_vec()));
		assert_eq!(uri_part.path, Some("/user1/feed".as_bytes().to_vec()));

		let raw_uri = "https://instagram.com/user1/feed".as_bytes().to_vec();
		let uri_part = URAuthParser::<Test>::try_parse(&raw_uri, &ClaimType::Domain).unwrap();
		assert_eq!(uri_part.scheme, "https://".as_bytes().to_vec());
		assert_eq!(uri_part.host, Some("instagram.com".as_bytes().to_vec()));
		assert_eq!(uri_part.sub_domain, Some("www.".as_bytes().to_vec()));
		assert_eq!(uri_part.path, Some("/user1/feed".as_bytes().to_vec()));

		// Full URI related to 'file' or 'dataset'
		let raw_uri = "newnal://file/cid".as_bytes().to_vec();
		let uri_part = URAuthParser::<Test>::try_parse(
			&raw_uri,
			&ClaimType::Contents {
				data_source: None,
				name: Default::default(),
				description: Default::default(),
			},
		)
		.unwrap();
		assert_eq!(uri_part.scheme, "newnal://".as_bytes().to_vec());
		assert_eq!(uri_part.host, Some("file".as_bytes().to_vec()));
		assert_eq!(uri_part.sub_domain, None);
		assert_eq!(uri_part.path, Some("/cid".as_bytes().to_vec()));

		// Partial URI related to 'file' or 'dataset'.
		// Scheme is set to 'newnal://'
		let raw_uri = "newnal://file/cid".as_bytes().to_vec();
		let uri_part = URAuthParser::<Test>::try_parse(
			&raw_uri,
			&ClaimType::Contents {
				data_source: None,
				name: Default::default(),
				description: Default::default(),
			},
		)
		.unwrap();
		assert_eq!(uri_part.scheme, "newnal://".as_bytes().to_vec());
		assert_eq!(uri_part.host, Some("file".as_bytes().to_vec()));
		assert_eq!(uri_part.sub_domain, None);
		assert_eq!(uri_part.path, Some("/cid".as_bytes().to_vec()));

		let raw_uri = "newnal://sub2.sub1.file/cid".as_bytes().to_vec();
		let uri_part = URAuthParser::<Test>::try_parse(
			&raw_uri,
			&ClaimType::Contents {
				data_source: None,
				name: Default::default(),
				description: Default::default(),
			},
		)
		.unwrap();
		println!("{:?}", sp_std::str::from_utf8(&uri_part.host.clone().unwrap()));
		assert_eq!(uri_part.scheme, "newnal://".as_bytes().to_vec());
		assert_eq!(uri_part.host, Some("file".as_bytes().to_vec()));
		assert_eq!(uri_part.sub_domain, Some("sub2.sub1.".as_bytes().to_vec()));
		assert_eq!(uri_part.path, Some("/cid".as_bytes().to_vec()));
	}

	// cargo t -p pallet-newnal --lib -- tests::parser_works --exact --nocapture
	#[test]
	fn parser_works() {
		assert_eq!(is_root_domain("http://instagram.com", ClaimType::Domain), true);
		assert_eq!(is_root_domain("https://instagram.com", ClaimType::Domain), true);
		assert_eq!(is_root_domain("https://www.instagram.com", ClaimType::Domain), true);
		assert_eq!(is_root_domain("https://sub2.sub1.www.instagram.com", ClaimType::Domain), false);
		assert_eq!(is_root_domain("ftp://www.instagram.com", ClaimType::Domain), true);
		assert_eq!(is_root_domain("ftp://instagram.com", ClaimType::Domain), true);
		assert_eq!(is_root_domain("ftp://sub2.sub1.www.instagram.com", ClaimType::Domain), false);
		assert_eq!(is_root_domain("smtp://sub2.sub1.www.instagram.com", ClaimType::Domain), false);
		assert_eq!(
			is_root_domain(
				"newnal://file/",
				ClaimType::Contents {
					data_source: None,
					name: Default::default(),
					description: Default::default()
				}
			),
			false
		);
		assert_eq!(
			is_root_domain(
				"newnal://file/cid",
				ClaimType::Contents {
					data_source: None,
					name: Default::default(),
					description: Default::default()
				}
			),
			true
		);
		assert_eq!(
			is_root_domain(
				"newnal://file/cid/1",
				ClaimType::Contents {
					data_source: None,
					name: Default::default(),
					description: Default::default()
				}
			),
			false
		);
		assert_eq!(
			is_root_domain(
				"newnal://sub1.file/cid/1",
				ClaimType::Contents {
					data_source: None,
					name: Default::default(),
					description: Default::default()
				}
			),
			false
		);
	}

	fn is_root_domain(uri: &str, claim_type: ClaimType) -> bool {
		let uri_part =
			URAuthParser::<Test>::try_parse(&uri.as_bytes().to_vec(), &claim_type).unwrap();
		println!("{}", uri_part);
		uri_part.is_root(&claim_type)
	}

	// cargo t -p pallet-newnal --lib -- types::tests::uri_part_eq_works --exact --nocapture
	#[test]
	fn uri_part_eq_works() {
		let uri_part1 = URIPart::new("https://".into(), None, Some("instagram.com".into()), None);
		let uri_part2 = URIPart::new("https://".into(), None, Some("instagram.com".into()), None);
		assert!(uri_part1 == uri_part2);
		let uri_part3 = URIPart::new(
			"https://".into(),
			None,
			Some("instagram.com".into()),
			Some("/coco".into()),
		);
		let uri_part4 = URIPart::new(
			"https://".into(),
			None,
			Some("instagram.com".into()),
			Some("/coco/1/2/3".into()),
		);
		let uri_part_any_path =
			URIPart::new("https://".into(), None, Some("instagram.com".into()), Some("/*".into()));
		assert!(uri_part3 == uri_part_any_path);
		assert!(uri_part4 == uri_part_any_path);
	}

	fn find_json_value(
		json_object: lite_json::JsonObject,
		field_name: &str,
		sub_field: Option<&str>,
	) -> Option<Vec<u8>> {
		let sub = sub_field.map_or("".into(), |s| s);
		let (_, json_value) = json_object
			.iter()
			.find(|(field, _)| field.iter().copied().eq(field_name.chars()))
			.unwrap();
		match json_value {
			lite_json::JsonValue::String(v) =>
				Some(v.iter().map(|c| *c as u8).collect::<Vec<u8>>()),
			lite_json::JsonValue::Object(v) => find_json_value(v.clone(), sub, None),
			_ => None,
		}
	}

	fn account_id_from_did_raw(mut raw: Vec<u8>) -> AccountId32 {
		let actual_owner_did: Vec<u8> = raw.drain(raw.len() - 48..raw.len()).collect();
		let mut output = bs58::decode(actual_owner_did).into_vec().unwrap();
		let temp: Vec<u8> = output.drain(1..33).collect();
		let mut raw_account_id = [0u8; 32];
		let buf = &temp[..raw_account_id.len()];
		raw_account_id.copy_from_slice(buf);
		raw_account_id.into()
	}

	#[test]
	fn json_parse_works() {
		use lite_json::{json_parser::parse_json, JsonValue};

		let json_string = r#"
            {
                "domain" : "website1.com",
                "adminDID" : "did:infra:ua:5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV",
                "challenge" : "__random_challenge_value__",
                "timestamp": "2023-07-28T10:17:21Z",
                "proof": {
                    "type": "Ed25519Signature2020",
                    "created": "2023-07-28T17:29:31Z",
                    "verificationMethod": "did:infra:ua:i3jr3...qW3dt#key-1",
                    "proofPurpose": "assertionMethod",
                    "proofValue": "gweEDz58DAdFfa9.....CrfFPP2oumHKtz"
                }
            } 
        "#;

		let json_data = parse_json(json_string).expect("Invalid!");
		let mut domain: Vec<u8> = vec![];
		let mut admin_did: Vec<u8> = vec![];
		let mut challenge: Vec<u8> = vec![];
		let mut timestamp: Vec<u8> = vec![];
		let mut proof_type: Vec<u8> = vec![];
		let mut proof: Vec<u8> = vec![];

		match json_data {
			JsonValue::Object(obj_value) => {
				domain = find_json_value(obj_value.clone(), "domain", None).unwrap();
				admin_did = find_json_value(obj_value.clone(), "adminDID", None).unwrap();
				challenge = find_json_value(obj_value.clone(), "challenge", None).unwrap();
				timestamp = find_json_value(obj_value.clone(), "timestamp", None).unwrap();
				proof_type = find_json_value(obj_value.clone(), "proof", Some("type")).unwrap();
				proof =
					find_json_value(obj_value.clone(), "proof".into(), Some("proofValue")).unwrap();
			},
			_ => {},
		}
		assert_eq!(domain, "website1.com".as_bytes().to_vec());
		assert_eq!(
			admin_did,
			"did:infra:ua:5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV"
				.as_bytes()
				.to_vec()
		);
		assert_eq!(challenge, "__random_challenge_value__".as_bytes().to_vec());
		assert_eq!(timestamp, "2023-07-28T10:17:21Z".as_bytes().to_vec());
		assert_eq!(proof_type, "Ed25519Signature2020".as_bytes().to_vec());
		assert_eq!(proof, "gweEDz58DAdFfa9.....CrfFPP2oumHKtz".as_bytes().to_vec());
		let account_id32 = account_id_from_did_raw(admin_did);
		println!("AccountId32 => {:?}", account_id32);
	}
}
