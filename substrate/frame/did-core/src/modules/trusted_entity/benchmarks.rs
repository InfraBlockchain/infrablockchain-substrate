use super::*;
use crate::{
	common::state_change::ToStateChange,
	did::{Did, DidSignature, UncheckedDidKey},
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::U256;
use sp_std::iter::once;
#[cfg(not(feature = "std"))]
use sp_std::prelude::*;

const MAX_TRUSTED_ENTITY: u32 = 1000;
const MAX_CONTROLLERS: u32 = 15;

fn dummy_authorizer<T: Limits>() -> Authorizer<T> {
	Authorizer { policy: Policy::one_of(once(Did([3; 32]))).unwrap(), add_only: false }
}

crate::bench_with_all_pairs! {
	with_pairs:
	add_issuer_sr25519 for sr25519, add_issuer_ed25519 for ed25519, add_issuer_secp256k1 for secp256k1 {
		{
			let r in 1 .. MAX_TRUSTED_ENTITY as u32;
		}
		let pair as Pair;
		let caller = whitelisted_caller();
		let did = Did([1; Did::BYTE_SIZE]);
		let public = pair.public();

		crate::did::Pallet::<T>::new_onchain_(
			did,
			vec![UncheckedDidKey::new_with_all_relationships(public)],
			Default::default(),
		).unwrap();

		let authorizer_id = AuthorizerId([1u8; 32]);
		let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).map(AuthorizerId).collect();
		let add_issuer_raw = AddIssuerRaw {
			 /// The authorizer on which to operate
			 authorizer_id: authorizer_id,
			 /// entity ids which will be added
			 entity_ids: entity_ids.clone(),
			 _marker: PhantomData
		};

		let add_issuer = AddIssuer::new_with_nonce(add_issuer_raw.clone(), 1u32.into());
		let sig = pair.sign(&add_issuer.to_state_change().encode());
		let signature = DidSignature::new(did, 1u32, sig);

		super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy: Policy::one_of(&[did]).unwrap(), add_only: false } }).unwrap();
	}: add_issuer(RawOrigin::Signed(caller), add_issuer_raw, vec![DidSignatureWithNonce { sig: signature, nonce: 1u32.into() }])
	verify {
		assert!(entity_ids
			.iter()
			.all(|id| Issuers::<T>::contains_key(authorizer_id, id)));
	}

	remove_issuer_sr25519 for sr25519, remove_issuer_ed25519 for ed25519, remove_issuer_secp256k1 for secp256k1 {
		{
			let r in 1 .. MAX_TRUSTED_ENTITY as u32;
		}
		let pair as Pair;
		let caller = whitelisted_caller();
		let did = Did([1; Did::BYTE_SIZE]);
		let public = pair.public();

		crate::did::Pallet::<T>::new_onchain_(
			did,
			vec![UncheckedDidKey::new_with_all_relationships(public)],
			Default::default(),
		).unwrap();

		let authorizer_id = AuthorizerId([1u8; 32]);
		let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).map(AuthorizerId).collect();

		super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy: Policy::one_of(&[did]).unwrap(), add_only: false } }).unwrap();

		crate::trusted_entity::Pallet::<T>::add_issuer_(
			AddIssuerRaw {
				/// The authorizer on which to operate
				authorizer_id: authorizer_id,
				/// entity ids which will be added
				entity_ids: entity_ids.clone(),
				_marker: PhantomData
		   },
			&mut dummy_authorizer()
		).unwrap();

		let remove_issuer_raw = RemoveIssuerRaw {
			/// The authorizer on which to operate
			authorizer_id: authorizer_id,
			/// entity ids which will be added
			entity_ids: entity_ids.clone(),
			_marker: PhantomData
		};

		let remove_issuer = RemoveIssuer::new_with_nonce(remove_issuer_raw.clone(), 1u32.into());
		let sig = pair.sign(&remove_issuer.to_state_change().encode());
		let signature = DidSignature::new(did, 1u32, sig);

	}: remove_issuer(RawOrigin::Signed(caller), remove_issuer_raw, vec![DidSignatureWithNonce { sig: signature, nonce: 1u32.into() }])
	verify {
		assert!(entity_ids
			.iter()
			.all(|id| !Issuers::<T>::contains_key(authorizer_id, id)));
	}

	add_verifier_sr25519 for sr25519, add_verifier_ed25519 for ed25519, add_verifier_secp256k1 for secp256k1 {
		{
			let r in 1 .. MAX_TRUSTED_ENTITY as u32;
		}
		let pair as Pair;
		let caller = whitelisted_caller();
		let did = Did([1; Did::BYTE_SIZE]);
		let public = pair.public();

		crate::did::Pallet::<T>::new_onchain_(
			did,
			vec![UncheckedDidKey::new_with_all_relationships(public)],
			Default::default(),
		).unwrap();

		let authorizer_id = AuthorizerId([1u8; 32]);
		let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).map(AuthorizerId).collect();
		let add_verifier_raw = AddVerifierRaw {
			 /// The authorizer on which to operate
			 authorizer_id: authorizer_id,
			 /// entity ids which will be added
			 entity_ids: entity_ids.clone(),
			 _marker: PhantomData
		};

		let add_verifier = AddVerifier::new_with_nonce(add_verifier_raw.clone(), 1u32.into());
		let sig = pair.sign(&add_verifier.to_state_change().encode());
		let signature = DidSignature::new(did, 1u32, sig);

		super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy: Policy::one_of(&[did]).unwrap(), add_only: false } }).unwrap();
	}: add_verifier(RawOrigin::Signed(caller), add_verifier_raw, vec![DidSignatureWithNonce { sig: signature, nonce: 1u32.into() }])
	verify {
		assert!(entity_ids
			.iter()
			.all(|id| Verifiers::<T>::contains_key(authorizer_id, id)));
	}

	remove_verifier_sr25519 for sr25519, remove_verifier_ed25519 for ed25519, remove_verifier_secp256k1 for secp256k1 {
		{
			let r in 1 .. MAX_TRUSTED_ENTITY as u32;
		}
		let pair as Pair;
		let caller = whitelisted_caller();
		let did = Did([1; Did::BYTE_SIZE]);
		let public = pair.public();

		crate::did::Pallet::<T>::new_onchain_(
			did,
			vec![UncheckedDidKey::new_with_all_relationships(public)],
			Default::default(),
		).unwrap();

		let authorizer_id = AuthorizerId([1u8; 32]);
		let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).map(AuthorizerId).collect();

		super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy: Policy::one_of(&[did]).unwrap(), add_only: false } }).unwrap();

		crate::trusted_entity::Pallet::<T>::add_verifier_(
			AddVerifierRaw {
				/// The authorizer on which to operate
				authorizer_id: authorizer_id,
				/// entity ids which will be added
				entity_ids: entity_ids.clone(),
				_marker: PhantomData
		   },
			&mut dummy_authorizer()
		).unwrap();

		let remove_verifier_raw = RemoveVerifierRaw {
			/// The authorizer on which to operate
			authorizer_id: authorizer_id,
			/// entity ids which will be added
			entity_ids: entity_ids.clone(),
			_marker: PhantomData
		};

		let remove_verifier = RemoveVerifier::new_with_nonce(remove_verifier_raw.clone(), 1u32.into());
		let sig = pair.sign(&remove_verifier.to_state_change().encode());
		let signature = DidSignature::new(did, 1u32, sig);

	}: remove_verifier(RawOrigin::Signed(caller), remove_verifier_raw, vec![DidSignatureWithNonce { sig: signature, nonce: 1u32.into() }])
	verify {
		assert!(entity_ids
			.iter()
			.all(|id| !Verifiers::<T>::contains_key(authorizer_id, id)));
	}

	remove_authorizer_sr25519 for sr25519, remove_authorizer_ed25519 for ed25519, remove_authorizer_secp256k1 for secp256k1 {
		let pair as Pair;
		let caller = whitelisted_caller();
		let public = pair.public();
		let did = Did([3 as u8; Did::BYTE_SIZE]);
		let authorizer_id = AuthorizerId([4 as u8; 32]);
		let authorizer = Authorizer {
			policy: Policy::one_of(once(did).chain((1..MAX_CONTROLLERS).map(U256::from).map(Into::into).map(Did)).collect::<Vec<_>>()).unwrap(),
			add_only: false,
		};
		let add_authorizer = AddAuthorizer {
			id: authorizer_id
			new_authorizer: authorizer.clone(),
		};
		let entity_ids: BTreeSet<_> = (0..100).map(|i| U256::from(i).into()).map(TrustedEntityId).collect();
		crate::did::Pallet::<T>::new_onchain_(
			did,
			vec![UncheckedDidKey::new_with_all_relationships(public)],
			Default::default(),
		).unwrap();

		super::Pallet::<T>::new_authorizer_(add_authorizer).unwrap();

		crate::trusted_entity::Pallet::<T>::add_issuer_(
			AddIssuerRaw {
				/// The authorizer on which to operate
				authorizer_id: authorizer_id,
				/// entity ids which will be added
				entity_ids: entity_ids.clone(),
				_marker: PhantomData
		   },
			&mut dummy_authorizer()
		).unwrap();

		let rem_authorizer_raw = RemoveAuthorizerRaw {
			authorizer_id: authorizer_id,
			_marker: PhantomData
		};
		let remove_authorizer = RemoveAuthorizer::new_with_nonce(rem_authorizer_raw.clone(), 1u32.into());
		let sig = pair.sign(&remove_authorizer.to_state_change().encode());
		let signature = DidSignature::new(did, 1u32, sig);
	}: remove_authorizer(RawOrigin::Signed(caller), remove_authorizer_raw, vec![DidSignatureWithNonce { sig: signature, nonce: 1u32.into() }])
	verify {
		assert!(Authorizers::<T>::get(authorizer_id).is_none());
	};

	standard:
	new_authorizer {
		let c in 1 .. MAX_CONTROLLERS;

		let caller = whitelisted_caller();
		let did = Did([3 as u8; Did::BYTE_SIZE]);
		let authorizer_id = AuthorizerId([4 as u8; 32]);
		let authorizer = Authorizer {
			policy: Policy::one_of(once(did).chain((1..c).map(U256::from).map(Into::into).map(Did)).collect::<Vec<_>>()).unwrap(),
			add_only: false,
		};
		let add_authorizer = AddAuthorizer {
			id: authorizer_id
			new_authorizer: authorizer.clone(),
		};

	}: new_authorizer(RawOrigin::Signed(caller), add_authorizer)
	verify {
		assert_eq!(Authorizers::<T>::get(authorizer_id).unwrap(), authorizer);
	}
}
