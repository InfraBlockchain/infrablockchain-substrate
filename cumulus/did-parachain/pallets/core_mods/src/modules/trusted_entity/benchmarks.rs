use super::*;
use crate::did::{Did, UncheckedDidKey};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use sp_core::U256;
use sp_std::{iter::once, prelude::*};
use system::RawOrigin;

const MAX_TRUSTED_ENTITY: u32 = 1000;
const MAX_CONTROLLERS: u32 = 15;

/// create a OneOf policy. Redefining from test as cannot import
pub fn oneof(dids: &[Did]) -> Policy {
    Policy::OneOf(dids.iter().cloned().collect())
}

crate::bench_with_all_pairs! {
    with_pairs:
    trusted_sr25519 for sr25519, trusted_ed25519 for ed25519 {
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

        let authorizer_id = [1u8; 32];
        let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).collect();
        let add_trusted_entity_raw = AddIssuerRaw {
             /// The authorizer on which to operate
            authorizer_id: authorizer_id,
            /// entity ids which will be added
            entity_ids: entity_ids.clone(),
            _marker: PhantomData
        };

        let add_trusted_entity = TrustedEntities::new_with_nonce(add_trusted_entity_raw.clone(), 1u32.into());
        let sig = pair.sign(&add_trusted_entity.to_state_change().encode());
        let signature = DidSignature::new(did, 1u32, sig);

        super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer {policy:
oneof(&[did]), add_only: false}}).unwrap(); 	}: add_trusted_entity(RawOrigin::Signed(caller), add_trusted_entity_raw,
vec![DidSigs { sig: signature, nonce: 1u32.into() }]) 	verify {
        assert!(entity_ids
            .iter()
            .all(|id| TrustedEntities::contains_key(authorizer_id, id)));
    }

    untrusted_sr25519 for sr25519, untrusted_ed25519 for ed25519 {
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

        let authorizer_id = [2u8; 32];
        let entity_ids: BTreeSet<_> = (0..r).map(|i| U256::from(i).into()).collect();

        super::Pallet::<T>::new_authorizer_(AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer {policy:
oneof(&[did]), add_only: false}}).unwrap();

        crate::trusted_entity::Pallet::<T>::add_issuer_(
            AddIssuerRaw {
                /// The authorizer on which to operate
               authorizer_id: authorizer_id,
               /// entity ids which will be added
               entity_ids: entity_ids.clone(),
               _marker: PhantomData
            },
            &mut Default::default(),
        ).unwrap();

        let remove_issuer = RemoveIssuerRaw {
            /// The authorizer on which to operate
           authorizer_id: authorizer_id,
           /// entity ids which will be added
           entity_ids: entity_ids.clone(),
           _marker: PhantomData
        };

        let remove_issuer = RemoveIssuer::new_with_nonce(remove_issuer.clone(), 1u32.into());
        let sig = pair.sign(&remove_issuer.to_state_change().encode());
        let signature = DidSignature::new(did, 1u32, sig);

    }: remove_issuer(RawOrigin::Signed(caller), remove_issuer, vec![DidSigs { sig: signature, nonce:
1u32.into() }]) 	verify {
        assert!(entity_ids
            .iter()
            .all(|id| !TrustedEntities::contains_key(authorizer_id, id)));
    }

    remove_authorizer_sr25519 for sr25519, remove_authorizer_ed25519 for ed25519 {
        let pair as Pair;
        let caller = whitelisted_caller();
        let public = pair.public();
        let did = Did([3 as u8; Did::BYTE_SIZE]);
        let authorizer_id = [4 as u8; 32];
        let authorizer = Authorizer {
            policy:
Policy::OneOf(once(did).chain((1..MAX_CONTROLLERS).map(U256::from).map(Into::into).map(Did)).
collect()), 			add_only: false,
        };
        let add_authorizer = AddAuthorizer {
            new_authorizer: authorizer.clone(),
            id: authorizer_id
        };
        let entity_ids: BTreeSet<_> = (0..100).map(|i| U256::from(i).into()).collect();
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
            &mut Default::default(),
        ).unwrap();

        let rem_authorizer_raw = RemoveAuthorizerRaw {
            authorizer_id: authorizer_id,
            _marker: PhantomData
        };
        let rem_reg = RemoveAuthorizer::new_with_nonce(rem_authorizer_raw.clone(), 1u32.into());
        let sig = pair.sign(&rem_reg.to_state_change().encode());
        let signature = DidSignature::new(did, 1u32, sig);
    }: remove_authorizer(RawOrigin::Signed(caller), rem_authorizer_raw, vec![DidSigs { sig: signature, nonce:
1u32.into() }]) 	verify {
        assert!(Authorizers::get(authorizer_id).is_none());
    };

    standard:
    new_authorizer {
        let c in 1 .. MAX_CONTROLLERS;

        let caller = whitelisted_caller();
        let did = Did([3 as u8; Did::BYTE_SIZE]);
        let authorizer_id = [4 as u8; 32];
        let authorizer = Authorizer {
            policy:
Policy::OneOf(once(did).chain((1..c).map(U256::from).map(Into::into).map(Did)).collect()),
            add_only: false,
        };
        let add_authorizer = AddAuthorizer {
            new_authorizer: authorizer.clone(),
            id: authorizer_id
        };

    }: new_authorizer(RawOrigin::Signed(caller), add_authorizer)
    verify {
        assert_eq!(Authorizers::get(authorizer_id).unwrap(), authorizer);
    }
}
