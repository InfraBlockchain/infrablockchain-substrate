#![cfg_attr(not(feature = "std"), no_std)]

use fixedstr::zstr;

use frame_support::{
	pallet_prelude::*,
	traits::{ConstU32, UnixTime},
	BoundedVec,
};

use frame_system::pallet_prelude::*;
use sp_core::{ecdsa, ed25519, sr25519, H256};
use sp_runtime::{
	traits::{BlakeTwo256, CheckedAdd, Hash, IdentifyAccount, Verify},
	AccountId32, MultiSignature, MultiSigner,
};
use sp_std::vec::Vec;
use xcm::latest::MultiAsset;

pub use pallet::*;

pub mod types;
pub use types::*;

pub mod parser;
pub use parser::*;

pub type RequestId = u64;
pub type TradeId = u64;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Parser for URAuth Pallet such as _challenge-value_, _URI_
		type URAuthParser: Parser<Self>;
		/// Time used for computing document creation.
		///
		/// It is guaranteed to start being called from the first `on_finalize`. Thus value at
		/// genesis is not used.
		type UnixTime: UnixTime;

		/// The current members of Oracle.
		type MaxOracleMembers: Get<u32>;

		/// URI list which should be verified by _Oracle_
		#[pallet::constant]
		type MaxURIByOracle: Get<u32>;

		/// Period for verifying to claim ownership
		#[pallet::constant]
		type VerificationPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type MaxRequest: Get<u32>;

		#[pallet::constant]
		type RandomnessEnabled: Get<bool>;

		/// The origin which may be used within _authorized_ call.
		/// **Root** can always do this.
		type AuthorizedOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	/// **Description:**
	///
	/// Store the `URAuthDoc` corresponding to the verified URI by Oracle nodes.
	/// The `URAuthDoc` contains definitions such as the DID of the owner for that URI and access
	/// permissions.
	///
	/// **Key:**
	///
	/// URI
	///
	/// **Value:**
	///
	/// URAuthDoc
	#[pallet::storage]
	pub type URAuthTree<T: Config> = StorageMap<_, Twox128, URI, URAuthDoc<T::AccountId>>;

	#[pallet::storage]
	pub type DIDs<T: Config> = StorageMap<_, Twox128, T::AccountId, DidDetails<T>>;

	/// **Description:**
	///
	/// Temporarily store the URIMetadata(owner_did and challenge_value) for the unverified URI in
	/// preparation for its verification.
	///
	/// **Key:**
	///
	/// URI
	///
	/// **Value:**
	///
	/// URIMetadata
	#[pallet::storage]
	#[pallet::unbounded]
	pub type Metadata<T: Config> = StorageMap<_, Twox128, URI, RequestMetadata>;

	#[pallet::storage]
	pub type RequestedURIs<T: Config> =
		StorageMap<_, Twox128, BlockNumberFor<T>, BoundedVec<URI, T::MaxRequest>>;

	#[pallet::storage]
	pub type DataSet<T: Config> = StorageMap<_, Twox128, URI, DataSetMetadata<AnyText>>;

	/// **Description:**
	///
	/// When validation is initiated by the Oracle node, store the submission status.
	/// For the requested URI, the Oracle node submits based on the Challenge Value and continues
	/// until it reaches a threshold value or higher.
	///
	/// **Key:**
	///
	/// URI
	///
	/// **Value:**
	///
	/// VerificationSubmission
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn uri_verification_info)]
	pub type URIVerificationInfo<T: Config> =
		StorageMap<_, Twox128, URI, VerificationSubmission<T>>;

	/// **Description:**
	///
	/// A random Challenge value is stored for the requested URI.
	/// The Randomness consists of a random value of 32 bytes.
	///
	/// **Key:**
	///
	/// URI
	///
	/// **Value:**
	///
	/// schnorrkel::Randomness
	#[pallet::storage]
	pub type ChallengeValue<T: Config> = StorageMap<_, Twox128, URI, Randomness>;
	/// **Description:**
	///
	/// The status of the URAuthDoc that has been requested for update on a specific field is stored
	/// in the form of UpdateDocStatus. URAuthDoc updates are only possible when the UpdateStatus is
	/// set to `Available`.
	///
	/// **Key:**
	///
	/// DocId
	///
	/// **Value:**
	///
	/// UpdateDocStatus
	#[pallet::storage]
	pub type URAuthDocUpdateStatus<T: Config> =
		StorageMap<_, Blake2_128Concat, DocId, UpdateDocStatus<T::AccountId>, ValueQuery>;

	/// **Description:**
	///
	/// Contains the AccountId information of the Oracle node.
	///
	/// **Value:**
	///
	/// BoundedVec<T::AccountId, T::MaxOracleMembers>
	#[pallet::storage]
	#[pallet::getter(fn oracle_members)]
	pub type OracleMembers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxOracleMembers>, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type URIByOracle<T: Config> = StorageValue<_, Vec<URIPart>, OptionQuery>;

	/// **Description:**
	///
	/// A counter used for generating the document id of the URAuthDoc.
	///
	/// **Value:**
	///
	/// URAuthDocCount
	#[pallet::storage]
	pub type Counter<T: Config> = StorageValue<_, URAuthDocCount, ValueQuery>;





	/// Data Purchase Request from Buyer
	#[pallet::storage]
	#[pallet::getter(fn data_purchase_requests)]
	pub type DataPurchaseRequests<T: Config> =
		StorageMap<_, Twox64Concat, RequestId, PurchaseRequestDetails<T::AccountId, BlockNumberFor<T>>, OptionQuery>;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash, Debug))]
	pub struct PurchaseRequestDetails<AccountId, BlockNumber> {
		pub buyer_id: AccountId,
		pub data_type: DataType,
		pub quantity: u64,
		// pub price_per_data: BalanceOf<T>,
		pub deadline: BlockNumber,
		pub status: PurchaseStatus,
	}

	#[pallet::storage]
	#[pallet::getter(fn data_transmits)]
	pub type DataTradeReceipts<T: Config> =
		StorageDoubleMap<_, Twox64Concat, RequestId, Twox64Concat, TradeId, TradeReceiptDetails<T::AccountId>, OptionQuery>;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash, Debug))]
	pub struct TradeReceiptDetails<AccountId> {
		pub seller_id: AccountId,
		pub data_type: DataType,
		pub quantity: u64,
	}

	#[pallet::storage]
	#[pallet::getter(fn purchase_agreements)]
	pub type PurchaseAgreements<T: Config> =
		StorageMap<_, Twox64Concat, RequestId, u64, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		URIFor<T>: Into<URI>,
		URIPartFor<T>: IsType<URIPart>,
	{
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let (r, w) = Self::handle_expired_requsted_uris(&n);
			T::DbWeight::get().reads_writes(r, w)
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub oracle_members: Vec<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { oracle_members: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let oracle_members: BoundedVec<T::AccountId, T::MaxOracleMembers> =
				self.oracle_members.clone().try_into().expect("Max Oracle members reached!");
			OracleMembers::<T>::put(oracle_members);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// URI is requested for its ownership.
		URAuthRegisterRequested {
			uri: URI,
		},
		/// `URAuthDoc` is registered on `URAuthTree`.
		URAuthTreeRegistered {
			claim_type: ClaimType,
			uri: URI,
			newnal_doc: URAuthDoc<T::AccountId>,
		},
		/// Oracle member has submitted its verification of challenge value.
		VerificationSubmitted {
			member: T::AccountId,
			digest: H256,
		},
		/// Result of `VerificationSubmission`.
		VerificationInfo {
			uri: URI,
			progress_status: VerificationSubmissionResult,
		},
		/// `URAuthDoc` has been updated for specific fiend.
		URAuthDocUpdated {
			update_doc_field: UpdateDocField<T::AccountId>,
			newnal_doc: URAuthDoc<T::AccountId>,
		},
		/// Update of `URAuthDoc` is in progress.
		UpdateInProgress {
			newnal_doc: URAuthDoc<T::AccountId>,
			update_doc_status: UpdateDocStatus<T::AccountId>,
		},
		/// List of `URIByOracle` has been added
		URIByOracleAdded,
		/// List of `URIByOracle` has been removed
		URIByOracleRemoved,
		/// Request of registering URI has been removed.
		Removed {
			uri: URI,
		},
		DataPurchaseFinalized {
			caller: T::AccountId,
			request_id: RequestId,
		},
		DataTradeExecuted {
			executor: T::AccountId,
			request_id: RequestId,
			trade_id: TradeId,
		},
		DataPurchaseRegistered {
			buyer: T::AccountId,
			request_id: RequestId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// May have overflowed its type(e.g Counter)
		Overflow,
		/// General error to do with the owner's proofs (e.g. signature).
		BadProof,
		/// The sending address is disabled or known to be invalid.
		BadSigner,
		/// General error on challenge value(e.g parsing json-string, different challenge value).
		BadChallengeValue,
		/// General error on requesting ownership(e.g URAuth Request, Challenge Value)
		BadRequest,
		/// General error on claiming ownership
		BadClaim,
		/// General error on URI
		BadURI,
		/// Size is over limit of `MAX_*`
		OverMaxSize,
		/// Error on converting raw-json to json-string.
		ErrorConvertToString,
		/// Error on converting raw-signature to concrete type signature.
		ErrorConvertToSignature,
		/// Error on decoding raw-did to bs58 encoding
		ErrorDecodeBs58,
		/// Error on converting `AccountId32` to `T::AccountId`
		ErrorDecodeAccountId,
		/// Error on decoding hex encoding string to actual string
		ErrorDecodeHex,
		/// General error on updating `URAuthDoc`(e.g Invalid UpdatedAt, Different update field)
		ErrorOnUpdateDoc,
		/// General error on updating `URAuthDocUpdateStatus`(e.g ProofMissing for updating
		/// `URAuthDoc`)
		ErrorOnUpdateDocStatus,
		/// General error on parsing (e.g URI, Challenge Value)
		ErrorOnParse,
		/// Error on some authorized calls which required origin as Oracle member
		NotOracleMember,
		/// Error when signer of signature is not `URAuthDoc` owner.
		NotURAuthDocOwner,
		/// Given URI is not URI which should be verified by Oracle
		NotURIByOracle,
		/// Given URI is not valid
		NotValidURI,
		/// General error on proof where it is required but it is not given.
		ProofMissing,
		/// Error when challenge value is not stored for requested URI.
		ChallengeValueMissing,
		/// Challenge value is not provided when `ChallengeValueConfig.randomness` is false.
		ChallengeValueNotProvided,
		/// When try to update `URAuthDoc` which has not registered.
		URAuthTreeNotRegistered,
		/// Oracle node has voted more than once
		AlreadySubmitted,
		/// Given URI has already registered on `URAuthTree`.
		AlreadyRegistered,
		/// When trying to add oracle member more than `T::MaxOracleMembers`
		MaxOracleMembers,
		/// Max number of request of URI per block has been reached.
		MaxRequest,
		/// When trying to update different field on `UpdateInProgress` field
		UpdateInProgress,
		/// Only URL is supported currently. General URI work in progress
		GeneralURINotSupportedYet,
		/// Error failed to the existing purchase request
		RequestDoesNotExist,
		/// Status of the purchase request is not active 
		RequestNotActive,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash))]
	pub enum PurchaseStatus {
		Active,
		Finished,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash))]
	pub enum DataType {
		Image,
		Json,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		URIFor<T>: Into<URI>,
		URIPartFor<T>: IsType<URIPart>,
		ClaimTypeFor<T>: From<ClaimType>,
		ChallengeValueFor<T>: Into<URAuthChallengeValue>,
	{
		// Description:
		// This transaction is for a domain owner to request ownership registration in the
		// URAuthTree. It involves verifying the signature for the data owner's DID on the given URI
		// and, if valid, generating a Challenge Value and storing it in the Storage.
		//
		// Origin:
		// ** Signed call **
		//
		// Params:
		// - uri: URI to be claimed its ownership
		// - owner_did: URI owner's DID
		// - challenge_value: Challenge value for verification
		// - signer: Entity who creates signature
		// - proof: Proof of URI's ownership
		//
		// Logic:
		// 1. Creation of a message for signature verification: `payload = (uri,
		//    owner_did).encode()`
		// 2. Signature verification
		// 3. If the signature is valid, generate a metadata(owner_did, challenge_value)
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn request_register_ownership(
			origin: OriginFor<T>,
			claim_type: ClaimType,
			uri: Vec<u8>,
			owner_did: Vec<u8>,
			challenge_value: Option<Randomness>,
			signer: MultiSigner,
			proof: MultiSignature,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let (maybe_register_uri, should_check_owner) =
				Self::check_uri(&claim_type, true, &uri, None)?;
			ensure!(
				URAuthTree::<T>::get(&maybe_register_uri).is_none(),
				Error::<T>::AlreadyRegistered
			);
			let bounded_uri: URI = uri.clone().try_into().map_err(|_| Error::<T>::OverMaxSize)?;
			let bounded_owner_did: OwnerDID =
				owner_did.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
			let (signer_acc, did_detail) = Self::verify_request_proof(
				&bounded_uri,
				&bounded_owner_did,
				&proof,
				signer,
				should_check_owner,
			)?;
			Self::try_add_requested_uris(&bounded_uri)?;
			let cv = Self::challenge_value(challenge_value)?;
			ChallengeValue::<T>::insert(&bounded_uri, cv);
			Metadata::<T>::insert(
				&bounded_uri,
				RequestMetadata::new(bounded_owner_did, cv, claim_type, maybe_register_uri),
			);
			DIDs::<T>::insert(&signer_acc, did_detail);

			Self::deposit_event(Event::<T>::URAuthRegisterRequested { uri: bounded_uri });

			Ok(())
		}

		// Description:
		// Oracle node will download `challenge-value.json` and call this transaction
		// , which is responsible for validating the challenge
		// To successfully register the `URAuthDoc` in the `URAuthTree`,
		// it's necessary that over 60% of the members in OracleMembers::<T> submit their
		// validations. Additionally, the approvals must meet this threshold to be considered valid
		// for the registration of the URAuthDoc in the URAuthTree.
		//
		// Origin:
		// ** Signed call **
		//
		// Params:
		// - challenge_value: Raw of challenge-value-json-string.
		//
		// Logic:
		// 1. Creation of a message for signature verification: `payload = (uri,
		//    owner_did).encode()`
		// 2. Signature verification
		// 3. If the signature is valid, generate a metadata(owner_did, challenge_value)
		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
		pub fn verify_challenge(origin: OriginFor<T>, challenge_json: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::oracle_members().contains(&who), Error::<T>::NotOracleMember);

			// Parse json
			let (sig, proof_type, raw_payload, uri, owner_did, challenge) =
				T::URAuthParser::parse_challenge_json(&challenge_json)?.into();

			// 1. OwnerDID of URI == Challenge Value's DID
			// 2. Verify signature
			let (owner, metadata) = Self::try_verify_challenge_value(
				sig,
				proof_type,
				raw_payload,
				&uri,
				&owner_did,
				challenge,
			)?;
			let member_count = Self::oracle_members().len();
			let mut vs = if let Some(vs) = URIVerificationInfo::<T>::get(&uri) {
				vs
			} else {
				VerificationSubmission::<T>::default()
			};
			let res = vs.submit(member_count, (who, BlakeTwo256::hash(&challenge_json)))?;
			Self::handle_verification_submission_result(&res, vs, &uri, owner, metadata)?;
			Self::deposit_event(Event::<T>::VerificationInfo { uri, progress_status: res });

			Ok(())
		}

		// Description:
		// After the registration in the URAuthTree is completed,
		// this transaction allows the owner to update the URAuthDoc.
		// Upon verifying the proof and if it's valid,
		// the transaction compares the weight values of owner DIDs and the threshold.
		// Finally, the updated URAuthDoc is stored in the URAuthTree.
		//
		// Origin:
		// ** Signed call **
		//
		// Params:
		// - uri: Key of `URAuthTree`
		// - update_doc_field: Which field of `URAuthDoc` to update
		// - updated_at: Timeframe of updating `URAuthDoc`
		// - proof: Proof of updating `URAuthDoc`
		//
		// Logic:
		// 1. Update the doc based on `UpdateDocStatus` & `UpdateDocField`
		// 2. Verify its given proof
		// 3. If valid, store on `URAuthTree` based on Multi DIDs weight and threshold
		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn update_newnal_doc(
			origin: OriginFor<T>,
			uri: URI,
			update_doc_field: UpdateDocField<T::AccountId>,
			updated_at: u128,
			proof: Option<Proof>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let (mut updated_newnal_doc, mut update_doc_status) =
				Self::try_update_newnal_doc(&uri, &update_doc_field, updated_at, proof.clone())?;
			let (owner, proof, did_detail) =
				Self::try_verify_newnal_doc_proof(&uri, &updated_newnal_doc, proof)?;
			Self::try_store_updated_newnal_doc(
				owner.clone(),
				proof,
				uri,
				&mut updated_newnal_doc,
				&mut update_doc_status,
				update_doc_field,
			)?;
			let owner_acc = Self::account_id_from_source(AccountIdSource::AccountId32(owner))?;
			DIDs::<T>::insert(&owner_acc, did_detail);
			Ok(())
		}

		// Description:
		// Claim ownership of `ClaimType::*`.
		// This doesn't require any challenge json verification.
		// Once signature is verified, URAuthDoc will be registered on URAuthTree.
		//
		// Origin:
		// ** Signed call **
		//
		// Params:
		// - claim_type: Type of claim to register on URAuthTree
		// - uri: URI to be claimed its ownership
		// - owner_did: URI owner's DID
		// - signer: Entity who creates signature
		// - proof: Proof of URI's ownership
		//
		// Logic:
		// 1. Verify signature
		// 2. Verify signer is one of the parent owners
		// 3. Once it is verified, create new URAuthDoc based on `claim_type`
		#[pallet::call_index(3)]
		#[pallet::weight(1_000)]
		pub fn claim_ownership(
			origin: OriginFor<T>,
			claim_type: ClaimType,
			uri: Vec<u8>,
			owner_did: Vec<u8>,
			signer: MultiSigner,
			proof: MultiSignature,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let maybe_parent_acc = Self::account_id_from_source(AccountIdSource::AccountId32(
				signer.clone().into_account(),
			))?;
			let (maybe_register_uri, should_check_owner) =
				Self::check_uri(&claim_type, false, &uri, Some(maybe_parent_acc))?;
			ensure!(
				URAuthTree::<T>::get(&maybe_register_uri).is_none(),
				Error::<T>::AlreadyRegistered
			);
			let bounded_uri: URI = uri.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
			let bounded_owner_did: OwnerDID =
				owner_did.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
			let (signer_acc, did_detail) = Self::verify_request_proof(
				&bounded_uri,
				&bounded_owner_did,
				&proof,
				signer,
				should_check_owner,
			)?;
			let owner =
				Self::account_id_from_source(AccountIdSource::DID(bounded_owner_did.to_vec()))?;
			let newnal_doc = match claim_type.clone() {
				ClaimType::Contents { data_source, name, description, .. } => {
					let bounded_name: AnyText =
						name.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
					let bounded_description: AnyText =
						description.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
					let bounded_data_source: Option<URI> = match data_source {
						Some(ds) => {
							let bounded: URI =
								ds.try_into().map_err(|_| Error::<T>::OverMaxSize)?;
							Some(bounded)
						},
						None => None,
					};
					DataSet::<T>::insert(
						&maybe_register_uri,
						DataSetMetadata::<AnyText>::new(bounded_name, bounded_description),
					);
					Self::new_newnal_doc(owner, None, bounded_data_source)?
				},
				_ => Self::new_newnal_doc(owner, None, None)?,
			};

			URAuthTree::<T>::insert(&maybe_register_uri, newnal_doc.clone());
			DIDs::<T>::insert(&signer_acc, did_detail);
			Self::deposit_event(Event::<T>::URAuthTreeRegistered {
				claim_type,
				uri: maybe_register_uri,
				newnal_doc,
			});

			Ok(())
		}

		// Description:
		// This transaction involves adding members of the Oracle node to the verification request
		// after downloading the Challenge Value.
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - who: Whom to be included as Oracle member
		#[pallet::call_index(4)]
		#[pallet::weight(1_000)]
		pub fn add_oracle_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			T::AuthorizedOrigin::ensure_origin(origin)?;

			OracleMembers::<T>::mutate(|m| {
				m.try_push(who).map_err(|_| Error::<T>::MaxOracleMembers)
			})?;

			Ok(())
		}

		// Description:
		// This transaction involves adding members of the Oracle node to the verification request
		// after downloading the Challenge Value.
		//
		// Origin:
		// ** Root(Authorized) privileged call **
		//
		// Params:
		// - who: Whom to be included as Oracle member
		#[pallet::call_index(5)]
		#[pallet::weight(1_000)]
		pub fn kick_oracle_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			T::AuthorizedOrigin::ensure_origin(origin)?;

			OracleMembers::<T>::mutate(|m| {
				m.try_push(who).map_err(|_| Error::<T>::MaxOracleMembers)
			})?;

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(1_000)]
		pub fn add_uri_by_oracle(
			origin: OriginFor<T>,
			claim_type: ClaimType,
			uri: Vec<u8>,
		) -> DispatchResult {
			T::AuthorizedOrigin::ensure_origin(origin)?;

			let uri_part: URIPart = T::URAuthParser::parse_uri(&uri, &claim_type)?.into();
			Self::check_claim_type(&uri_part, &claim_type)?;
			URIByOracle::<T>::try_mutate_exists(|uri_parts| -> DispatchResult {
				let mut new = uri_parts.clone().map_or(Vec::new(), |v| v.to_vec());
				new.push(uri_part.clone());
				*uri_parts = Some(new.try_into().map_err(|_| Error::<T>::OverMaxSize)?);
				Ok(())
			})?;
			Self::deposit_event(Event::<T>::URIByOracleAdded);
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(1_000)]
		pub fn remove_uri_by_oracle(
			origin: OriginFor<T>,
			claim_type: ClaimType,
			uri: Vec<u8>,
		) -> DispatchResult {
			T::AuthorizedOrigin::ensure_origin(origin)?;

			let uri_part: URIPart = T::URAuthParser::parse_uri(&uri, &claim_type)?.into();
			let mut is_removed = true;
			URIByOracle::<T>::try_mutate_exists(|uri_parts| -> DispatchResult {
				if let Some(v) = uri_parts {
					let new = v
						.into_iter()
						.filter(|u| *u != &uri_part)
						.map(|u| u.clone())
						.collect::<Vec<URIPart>>();
					*uri_parts = Some(new.try_into().map_err(|_| Error::<T>::OverMaxSize)?);
				} else {
					is_removed = false;
				}
				Ok(())
			})?;
			if is_removed {
				Self::deposit_event(Event::<T>::URIByOracleRemoved);
			}
			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(1_000)]
		pub fn register_data_purchase(
			origin: OriginFor<T>,
			data_type: DataType,
			quantity: u64,
			// price_per_data: BalanceOf<T>,
			deadline: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			let buyer = ensure_signed(origin)?;

			// Generating a unique request ID
			let request_id = 0;
			// let request_id = Self::generate_request_id(&buyer, &data_type, &quantity);

			// Create a purchase request structure
			let new_request = PurchaseRequestDetails {
				buyer_id: buyer.clone(),
				data_type,
				quantity,
				// price_per_data,
				deadline,
				status: PurchaseStatus::Active, // Assume PurchaseStatus enum is defined somewhere
			};

			// Insert the new request into storage
			DataPurchaseRequests::<T>::insert(request_id, new_request);

			// Initialize the purchase agreement count for this request
			PurchaseAgreements::<T>::insert(request_id, 0);

			// Logic to lock the specified amount of tokens in a specific address
			// Implement token locking mechanism here...

			// Emit an event for successful registration
			Self::deposit_event(Event::<T>::DataPurchaseRegistered{buyer, request_id});

			Ok(().into())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(10_000)]
		pub fn execute_data_trade(
			origin: OriginFor<T>,
			request_id: RequestId,
			trade_id: TradeId, // A unique identifier for the trade
		) -> DispatchResultWithPostInfo {
			let executor = ensure_signed(origin)?;

			// Ensure the purchase request is valid and active
			let purchase_request = DataPurchaseRequests::<T>::get(&request_id)
				.ok_or(Error::<T>::RequestDoesNotExist)?;
			ensure!(
				purchase_request.status == PurchaseStatus::Active,
				Error::<T>::RequestNotActive
			);

			// Logic for transferring tokens from the buyer's escrow to the seller
			// Implement token transfer mechanism here...

			// Record the trade details in the DataTradeReceipts storage
			let new_trade_receipt = TradeReceiptDetails {
				seller_id: executor.clone(), // or however you determine the seller
				data_type: purchase_request.data_type,
				quantity: 1, 
			};

			DataTradeReceipts::<T>::insert(request_id, trade_id, new_trade_receipt);

			// Emit an event for successful trade execution
			Self::deposit_event(Event::<T>::DataTradeExecuted{executor, request_id, trade_id});

			Ok(().into())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(1_000)]
		pub fn finalize_data_purchase(
			origin: OriginFor<T>,
			request_id: RequestId,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			// Retrieve the purchase request
			let mut purchase_request = DataPurchaseRequests::<T>::get(&request_id)
				.ok_or(Error::<T>::RequestDoesNotExist)?;

			ensure!(
				purchase_request.status == PurchaseStatus::Active,
				Error::<T>::RequestNotActive
			);

			// Update the status to Completed
			purchase_request.status = PurchaseStatus::Finished;
			DataPurchaseRequests::<T>::insert(&request_id, purchase_request);

			// Emit an event for purchase finalization
			Self::deposit_event(Event::<T>::DataPurchaseFinalized{caller, request_id});
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T>
where
	URIFor<T>: Into<URI>,
	URIPartFor<T>: IsType<URIPart>,
{
	/// 16 bytes uuid based on `URAuthDocCount`
	fn doc_id() -> Result<DocId, DispatchError> {
		let count = Counter::<T>::get();
		Counter::<T>::try_mutate(|c| -> DispatchResult {
			*c = c.checked_add(1).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;
		let b = count.to_le_bytes();

		Ok(nuuid::Uuid::from_bytes(b).to_bytes())
	}

	fn unix_time() -> u128 {
		<T as Config>::UnixTime::now().as_millis()
	}

	fn challenge_value(challenge_value: Option<Randomness>) -> Result<Randomness, DispatchError> {
		let cv = if T::RandomnessEnabled::get() {
			Default::default()
		} else {
			challenge_value.ok_or(Error::<T>::ChallengeValueNotProvided)?
		};
		Ok(cv)
	}

	fn try_increase_nonce(acc: &T::AccountId) -> Result<DidDetails<T>, DispatchError> {
		let mut did_detail = DIDs::<T>::get(acc).map_or(DidDetails::default(), |v| v);
		did_detail.try_increase_nonce()?;
		Ok(did_detail)
	}

	fn try_add_requested_uris(uri: &URI) -> DispatchResult {
		let expire = <frame_system::Pallet<T>>::block_number() + T::VerificationPeriod::get();
		RequestedURIs::<T>::try_mutate_exists(expire, |uris| -> DispatchResult {
			let mut new = uris.clone().map_or(Default::default(), |v| v);
			new.try_push(uri.clone()).map_err(|_| Error::<T>::MaxRequest)?;
			*uris = Some(new);
			Ok(())
		})?;
		Ok(())
	}

	fn check_claim_type(uri_part: &URIPart, claim_type: &ClaimType) -> DispatchResult {
		match claim_type {
			ClaimType::Domain => ensure!(uri_part.host.is_some(), Error::<T>::BadClaim),
			_ => {
				ensure!(uri_part.scheme == "newnal://".as_bytes().to_vec(), Error::<T>::BadClaim);
			},
		}
		Ok(())
	}

	fn check_uri(
		claim_type: &ClaimType,
		is_oracle: bool,
		raw_uri: &Vec<u8>,
		maybe_parent_acc: Option<T::AccountId>,
	) -> Result<(URI, bool), DispatchError> {
		let parsed_uri_part: URIPart = T::URAuthParser::parse_uri(raw_uri, claim_type)?.into();
		let mut should_check_owner: bool = true;
		Self::check_claim_type(&parsed_uri_part, claim_type)?;
		let uri = if parsed_uri_part.is_root(claim_type) {
			let (_, root_uri) = parsed_uri_part.full_uri();
			root_uri
		} else {
			if is_oracle {
				Self::check_uri_by_oracle(parsed_uri_part)?
			} else {
				should_check_owner = false;
				Self::check_parent_owner(
					raw_uri.clone(),
					&maybe_parent_acc.ok_or(Error::<T>::BadClaim)?,
					&claim_type,
				)?
			}
		};
		Ok((uri.try_into().map_err(|_| Error::<T>::OverMaxSize)?, should_check_owner))
	}

	/// Check owner of given 'uri'. Parse the given uri
	/// and check whether given owner is one of the owner of the parent_uris
	///
	/// ## Example
	///
	/// - uri: "sub2.sub1.example.com/path1"
	///
	/// - base: "example.com"
	///
	/// - parent_uri: ["(sub2.sub1.example.com, owner1)", "(sub1.example.com, owner2)",
	///   "(example.com, owner3)"]
	///
	/// Check `maybe_owner == owner1?` -> `maybe_owner == owner2?` -> `maybe_owner == owner3?`.
	/// If not, return `Error::<T>::NotURAuthDocOwner`
	fn check_parent_owner(
		raw_uri: Vec<u8>,
		maybe_owner: &T::AccountId,
		claim_type: &ClaimType,
	) -> Result<Vec<u8>, DispatchError> {
		let uris = <URAuthParser<T> as Parser<T>>::parse_parent_uris(&raw_uri, &claim_type)?;
		for uri in uris {
			if let Some(newnal_doc) = URAuthTree::<T>::get(&uri) {
				if newnal_doc.is_owner(maybe_owner) {
					return Ok(raw_uri)
				}
			}
		}
		Err(Error::<T>::NotURAuthDocOwner.into())
	}

	fn check_uri_by_oracle(parsed_uri_part: URIPart) -> Result<Vec<u8>, DispatchError> {
		let mut temp: Option<Vec<u8>> = None;
		if let Some(uri_parts) = URIByOracle::<T>::get() {
			for uri_part in uri_parts {
				if parsed_uri_part == uri_part {
					let (_, full_uri) = parsed_uri_part.full_uri();
					temp = Some(full_uri);
					break
				}
			}
			temp.ok_or(Error::<T>::BadClaim.into())
		} else {
			return Err(Error::<T>::NotURIByOracle.into())
		}
	}

	fn new_newnal_doc(
		owner_did: T::AccountId,
		asset: Option<MultiAsset>,
		data_source: Option<URI>,
	) -> Result<URAuthDoc<T::AccountId>, DispatchError> {
		let doc_id = Self::doc_id()?;
		Ok(URAuthDoc::new(
			doc_id,
			MultiDID::new(owner_did, 1),
			Self::unix_time(),
			asset,
			data_source,
		))
	}

	/// Verify `request` signature
	///
	/// 1. Check whether owner and signer are same
	/// 2. Check the signature
	fn verify_request_proof(
		uri: &URI,
		owner_did: &OwnerDID,
		signature: &MultiSignature,
		signer: MultiSigner,
		should_check_owner: bool,
	) -> Result<(T::AccountId, DidDetails<T>), DispatchError> {
		// Check whether account id of owner did and signer are same
		let signer_account_id = Self::account_id_from_source(AccountIdSource::AccountId32(
			signer.clone().into_account(),
		))?;

		let did_detail = Self::try_increase_nonce(&signer_account_id)?;
		let newnal_signed_payload =
			URAuthSignedPayload::<T::AccountId, BlockNumberFor<T>>::Request {
				uri: uri.clone(),
				owner_did: owner_did.clone(),
				nonce: did_detail.nonce(),
			};

		if should_check_owner {
			Self::check_is_valid_owner(owner_did, &signer_account_id)?;
		}

		// Check signature
		if !newnal_signed_payload
			.using_encoded(|payload| signature.verify(payload, &signer.into_account()))
		{
			return Err(Error::<T>::BadProof.into())
		}

		Ok((signer_account_id, did_detail))
	}

	fn handle_expired_requsted_uris(n: &BlockNumberFor<T>) -> (u64, u64) {
		let mut r: u64 = 1;
		let mut w: u64 = 1;
		if let Some(uris) = RequestedURIs::<T>::get(n) {
			for uri in uris.iter() {
				Self::remove_all_uri_related(uri);
				r += 1;
				w += 3;
			}
		}
		RequestedURIs::<T>::remove(n);
		(r, w + 1)
	}

	/// Handle the result of _challenge value_ verification based on `VerificationSubmissionResult`
	fn handle_verification_submission_result(
		res: &VerificationSubmissionResult,
		verification_submission: VerificationSubmission<T>,
		uri: &URI,
		owner_did: T::AccountId,
		metadata: RequestMetadata,
	) -> Result<(), DispatchError> {
		match res {
			VerificationSubmissionResult::Complete => {
				let RequestMetadata { claim_type, maybe_register_uri, .. } = metadata;
				let newnal_doc = Self::new_newnal_doc(owner_did, None, None)?;
				URAuthTree::<T>::insert(&maybe_register_uri, newnal_doc.clone());
				Self::remove_all_uri_related(&uri);
				Self::deposit_event(Event::<T>::URAuthTreeRegistered {
					claim_type,
					uri: maybe_register_uri,
					newnal_doc,
				})
			},
			VerificationSubmissionResult::Tie => Self::remove_all_uri_related(&uri),
			VerificationSubmissionResult::InProgress =>
				URIVerificationInfo::<T>::insert(&uri, verification_submission),
		}

		Ok(())
	}

	/// Verify _challenge value_
	///
	/// 1. Check given signature
	/// 2. Check whether `signer` and `owner` are identical
	/// 3. Check whether `given` challenge value is same with `on-chain` challenge value
	fn try_verify_challenge_value(
		sig: Vec<u8>,
		proof_type: Vec<u8>,
		raw_payload: Vec<u8>,
		uri: &URI,
		owner_did: &OwnerDID,
		challenge: Vec<u8>,
	) -> Result<(T::AccountId, RequestMetadata), DispatchError> {
		let multi_sig = Self::raw_signature_to_multi_sig(&proof_type, &sig)?;
		let signer = Self::account_id32_from_raw_did(owner_did.to_vec())?;
		if !multi_sig.verify(&raw_payload[..], &signer) {
			return Err(Error::<T>::BadProof.into())
		}
		let uri_metadata = Metadata::<T>::get(uri).ok_or(Error::<T>::BadRequest)?;
		let signer = Self::account_id_from_source(AccountIdSource::DID(owner_did.to_vec()))?;
		Self::check_is_valid_owner(&uri_metadata.owner_did, &signer)
			.map_err(|_| Error::<T>::BadSigner)?;
		Self::check_challenge_value(uri, challenge)?;
		Ok((signer, uri_metadata))
	}

	/// Check whether `owner` and `signer` are identical
	fn check_is_valid_owner(
		raw_owner_did: &Vec<u8>,
		signer: &T::AccountId,
	) -> Result<(), DispatchError> {
		let owner_account_id =
			Self::account_id_from_source(AccountIdSource::DID(raw_owner_did.clone()))?;
		ensure!(&owner_account_id == signer, Error::<T>::BadSigner);
		Ok(())
	}

	/// Check whether `given` challenge value is same with `on-chain` challenge value
	///
	/// ## Errors
	///
	/// `ChallengeValueMissing`
	///
	/// Challenge value for given _uri_ is not stored
	///
	/// - `BadChallengeValue`
	///
	/// Given challenge value and on-chain challenge value are not identical
	fn check_challenge_value(uri: &URI, challenge: Vec<u8>) -> Result<(), DispatchError> {
		let cv = ChallengeValue::<T>::get(&uri).ok_or(Error::<T>::ChallengeValueMissing)?;
		ensure!(challenge == cv.to_vec(), Error::<T>::BadChallengeValue);
		Ok(())
	}

	/// Update the `URAuthDoc` based on `UpdateDocField`.
	/// If it is first time requested, `UpdateStatus` would be `Available`.
	/// Otherwise, `InProgress { .. }`.
	///
	/// ## Errors
	/// `ProofMissing`
	/// `URAuthTreeNotRegistered`
	fn try_update_newnal_doc(
		uri: &URI,
		update_doc_field: &UpdateDocField<T::AccountId>,
		updated_at: u128,
		maybe_proof: Option<Proof>,
	) -> Result<(URAuthDoc<T::AccountId>, UpdateDocStatus<T::AccountId>), DispatchError> {
		let _ = maybe_proof.ok_or(Error::<T>::ProofMissing)?;
		let mut newnal_doc =
			URAuthTree::<T>::get(uri).ok_or(Error::<T>::URAuthTreeNotRegistered)?;
		let mut update_doc_status = URAuthDocUpdateStatus::<T>::get(&newnal_doc.id);
		Self::do_try_update_doc(
			&mut newnal_doc,
			&mut update_doc_status,
			update_doc_field,
			updated_at,
		)?;

		Ok((newnal_doc, update_doc_status))
	}

	/// Try to store _updated_ `URAuthDoc` on `URAuthTree::<T>`.
	///
	/// Check whether _did_weight_ is greater of equal to _remaining_threshold_.
	/// If it is bigger, _1. remove all previous proofs 2. and store on `URAuthTree::<T>`._
	/// Otherwise, update `URAuthDocUpdateStatus`.
	fn handle_updated_newnal_doc(
		signer: AccountId32,
		proof: Proof,
		uri: URI,
		newnal_doc: &mut URAuthDoc<T::AccountId>,
		update_doc_status: &mut UpdateDocStatus<T::AccountId>,
		update_doc_field: UpdateDocField<T::AccountId>,
	) -> Result<(), DispatchError> {
		let multi_did = newnal_doc.get_multi_did();
		let account_id = Pallet::<T>::account_id_from_source(AccountIdSource::AccountId32(signer))?;
		let did_weight =
			multi_did.get_did_weight(&account_id).ok_or(Error::<T>::NotURAuthDocOwner)?;
		let remaining_threshold = update_doc_status.remaining_threshold;
		update_doc_status
			.handle_in_progress(did_weight, update_doc_field.clone(), proof)
			.map_err(|_| Error::<T>::ErrorOnUpdateDocStatus)?;
		if did_weight >= remaining_threshold {
			let new_proofs = update_doc_status.get_proofs();
			newnal_doc.handle_proofs(new_proofs);
			URAuthTree::<T>::insert(uri, newnal_doc.clone());
			URAuthDocUpdateStatus::<T>::remove(newnal_doc.id);
			Pallet::<T>::deposit_event(Event::<T>::URAuthDocUpdated {
				update_doc_field,
				newnal_doc: newnal_doc.clone(),
			});
		} else {
			URAuthDocUpdateStatus::<T>::insert(newnal_doc.id, update_doc_status.clone());
			Pallet::<T>::deposit_event(Event::<T>::UpdateInProgress {
				newnal_doc: newnal_doc.clone(),
				update_doc_status: update_doc_status.clone(),
			});
		}
		Ok(())
	}

	/// Try verify proof of _updated_ `URAuthDoc`.
	///
	/// ## Errors
	/// `ProofMissing` : Proof is not provided
	///
	/// `NotURAuthDocOwner` : If signer is not owner of `URAuthDoc`
	///
	/// `BadProof` : Signature is not valid
	fn try_verify_newnal_doc_proof(
		uri: &URI,
		newnal_doc: &URAuthDoc<T::AccountId>,
		proof: Option<Proof>,
	) -> Result<(AccountId32, Proof, DidDetails<T>), DispatchError> {
		let (owner_did, sig) = match proof.clone().ok_or(Error::<T>::ProofMissing)? {
			Proof::ProofV1 { did, proof } => (did, proof),
		};
		let owner_account = Self::account_id_from_source(AccountIdSource::DID(owner_did.to_vec()))?;
		if !newnal_doc.multi_owner_did.is_owner(&owner_account) {
			return Err(Error::<T>::NotURAuthDocOwner.into())
		}
		let did_detail = Self::try_increase_nonce(&owner_account)?;
		let payload = URAuthSignedPayload::<T::AccountId, BlockNumberFor<T>>::Update {
			uri: uri.clone(),
			newnal_doc: newnal_doc.clone(),
			owner_did: owner_did.clone(),
			nonce: did_detail.nonce(),
		};
		let signer = Pallet::<T>::account_id32_from_raw_did(owner_did.to_vec())?;
		if !payload.using_encoded(|m| sig.verify(m, &signer)) {
			return Err(Error::<T>::BadProof.into())
		}

		Ok((signer, proof.expect("Already checked!"), did_detail))
	}

	/// Try to store _updated_newnal_doc_ on `URAuthTree::<T>` based on `URAuthDocStatus`
	fn try_store_updated_newnal_doc(
		signer: AccountId32,
		proof: Proof,
		uri: URI,
		newnal_doc: &mut URAuthDoc<T::AccountId>,
		update_doc_status: &mut UpdateDocStatus<T::AccountId>,
		updated_doc_field: UpdateDocField<T::AccountId>,
	) -> Result<(), DispatchError> {
		Self::handle_updated_newnal_doc(
			signer,
			proof,
			uri,
			newnal_doc,
			update_doc_status,
			updated_doc_field,
		)?;

		Ok(())
	}

	/// Convert raw signature type to _concrete(`MultiSignature`)_ signature type
	///
	/// ## Error
	/// `ErrorConvertToSignature`
	fn raw_signature_to_multi_sig(
		proof_type: &Vec<u8>,
		sig: &Vec<u8>,
	) -> Result<MultiSignature, DispatchError> {
		let zstr_proof = zstr::<128>::from_raw(proof_type).to_ascii_lower();
		let proof_type = zstr_proof.to_str();
		if proof_type.contains("ed25519") {
			let sig = ed25519::Signature::try_from(&sig[..])
				.map_err(|_| Error::<T>::ErrorConvertToSignature)?;
			Ok(sig.into())
		} else if proof_type.contains("sr25519") {
			let sig = sr25519::Signature::try_from(&sig[..])
				.map_err(|_| Error::<T>::ErrorConvertToSignature)?;
			Ok(sig.into())
		} else {
			let sig = ecdsa::Signature::try_from(&sig[..])
				.map_err(|_| Error::<T>::ErrorConvertToSignature)?;
			Ok(sig.into())
		}
	}

	/// Remove all `URI` related data when `VerificationSubmissionResult::Complete` or
	/// `VerificationSubmissionResult::Tie`
	///
	/// ## Changes
	/// `URIMetadata`, `URIVerificationInfo`, `ChallengeValue`
	fn remove_all_uri_related(uri: &URI) {
		Metadata::<T>::remove(uri);
		URIVerificationInfo::<T>::remove(uri);
		ChallengeValue::<T>::remove(uri);

		Self::deposit_event(Event::<T>::Removed { uri: uri.clone() })
	}

	/// Try to convert some id sources to `T::AccountId` based on `AccountIdSource::DID` or
	/// `AccountIdSource::AccountId32`
	///
	/// ## Error
	/// `BadChallengeValue`: Since AccountId is length of _48_. If is shorter than 48, we assume it
	/// is invalid.
	///
	/// `ErrorDecodeAccountId` : Fail on convert from `AccountId32` to `T::AccountId`
	fn account_id_from_source(source: AccountIdSource) -> Result<T::AccountId, DispatchError> {
		let account_id32 = match source {
			AccountIdSource::DID(mut raw_owner_did) => {
				let byte_len = raw_owner_did.len();
				if byte_len < 48 {
					return Err(Error::<T>::BadChallengeValue.into())
				}
				let actual_owner_did: Vec<u8> =
					raw_owner_did.drain(byte_len - 48..byte_len).collect();
				let mut output: Vec<u8> = bs58::decode(actual_owner_did)
					.into_vec()
					.map_err(|_| Error::<T>::ErrorDecodeBs58)?;
				let temp: Vec<u8> = output.drain(1..33).collect();
				let mut raw_account_id = [0u8; 32];
				let buf = &temp[..raw_account_id.len()];
				raw_account_id.copy_from_slice(buf);
				raw_account_id.into()
			},
			AccountIdSource::AccountId32(id) => id,
		};

		let account_id = T::AccountId::decode(&mut account_id32.as_ref())
			.map_err(|_| Error::<T>::ErrorDecodeAccountId)?;
		Ok(account_id)
	}

	/// Try to convert from `raw_owner_did` to `AccountId32`
	///
	/// ## Error
	/// `BadChallengeValue`, `ErrorDecodeBs58`
	fn account_id32_from_raw_did(mut raw_owner_did: Vec<u8>) -> Result<AccountId32, DispatchError> {
		let byte_len = raw_owner_did.len();
		if byte_len < 48 {
			return Err(Error::<T>::BadChallengeValue.into())
		}
		let actual_owner_did: Vec<u8> = raw_owner_did.drain(byte_len - 48..byte_len).collect();
		let mut output = bs58::decode(actual_owner_did)
			.into_vec()
			.map_err(|_| Error::<T>::ErrorDecodeBs58)?;
		let temp: Vec<u8> = output.drain(1..33).collect();
		let mut raw_account_id = [0u8; 32];
		let buf = &temp[..raw_account_id.len()];
		raw_account_id.copy_from_slice(buf);

		Ok(raw_account_id.into())
	}
}

impl<T: Config> Pallet<T> {
	/// Check whether `updated_at` is greater than `prev_updated_at`
	///
	/// ## Error
	/// `ErrorOnUpdateDoc`
	fn check_valid_updated_at(prev: u128, now: u128) -> Result<(), DispatchError> {
		if !prev <= now {
			return Err(Error::<T>::ErrorOnUpdateDoc.into())
		}
		Ok(())
	}

	/// Try handle for updating `URAuthDocStatus`
	///
	/// ## Error
	/// `ErrorOnUpdateDoc` : Try to update on different field
	fn handle_update_doc_status(
		update_doc_status: &mut UpdateDocStatus<T::AccountId>,
		update_doc_field: &UpdateDocField<T::AccountId>,
		threshold: DIDWeight,
	) -> Result<(), DispatchError> {
		match &update_doc_status.status {
			UpdateStatus::Available => {
				update_doc_status.handle_available(threshold, update_doc_field.clone());
			},
			UpdateStatus::InProgress { field, .. } =>
				if field != update_doc_field {
					return Err(Error::<T>::ErrorOnUpdateDoc.into())
				},
		}

		Ok(())
	}

	/// Try to update doc and return _Err_ if any.
	///
	/// ## Errors
	///
	/// `ErrorOnUpdateDoc`
	/// - _updated_at_ is less than _pref_updated_at_
	/// - Try to update on different field
	/// - Threshold is bigger than sum of _multi_dids'_ weight
	pub fn do_try_update_doc(
		newnal_doc: &mut URAuthDoc<T::AccountId>,
		update_doc_status: &mut UpdateDocStatus<T::AccountId>,
		update_doc_field: &UpdateDocField<T::AccountId>,
		updated_at: u128,
	) -> Result<(), DispatchError> {
		let prev_updated_at = newnal_doc.updated_at;
		Self::check_valid_updated_at(prev_updated_at, updated_at)?;
		Self::handle_update_doc_status(
			update_doc_status,
			update_doc_field,
			newnal_doc.get_threshold(),
		)?;

		newnal_doc.update_doc(update_doc_field.clone(), updated_at).map_err(|e| {
			log::warn!(" 🚨 Error on update newnal_doc {:?} 🚨", e);
			Error::<T>::ErrorOnUpdateDoc
		})?;

		Ok(())
	}

	/// ## **RUNTIME API METHOD**
	///
	/// Return _updated_ `Some(URAuthDoc)`.
	/// Return `None` if given `URI` is not registered on `URAuthTree`
	/// or `ErrorOnUpdateDoc`.
	pub fn get_updated_doc(
		uri: URI,
		update_doc_field: UpdateDocField<T::AccountId>,
		updated_at: u128,
	) -> Option<URAuthDoc<T::AccountId>> {
		if let Some(mut newnal_doc) = URAuthTree::<T>::get(&uri) {
			match newnal_doc.update_doc(update_doc_field, updated_at) {
				Ok(_) => Some(newnal_doc.clone()),
				Err(_) => None,
			}
		} else {
			None
		}
	}
}
