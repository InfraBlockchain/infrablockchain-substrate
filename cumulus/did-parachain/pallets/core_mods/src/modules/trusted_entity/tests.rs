use super::*;
use crate::{test_common::*, util::WithNonce, Action, ToStateChange};
use alloc::collections::BTreeMap;
use core::{iter::once, marker::PhantomData};
use frame_support::assert_noop;
use sp_core::{sr25519, U256};

pub fn get_pauth<A: Action<Test> + Clone>(
    action: &A,
    signers: &[(Did, &sr25519::Pair)],
) -> Vec<DidSigs<Test>>
where
    WithNonce<Test, A>: ToStateChange<Test>,
{
    signers
        .iter()
        .map(|(did, kp)| {
            let did_detail = DIDModule::onchain_did_details(&did).unwrap();
            let next_nonce = did_detail.next_nonce().unwrap();
            let sp = WithNonce::<Test, _>::new_with_nonce(action.clone(), next_nonce);
            let sig =
                did_sig_on_bytes::<Test, _>(&sp.to_state_change().encode(), &kp, did.clone(), 1);
            DidSigs {
                sig,
                nonce: next_nonce,
            }
        })
        .collect()
}

pub fn get_nonces(signers: &[(Did, &sr25519::Pair)]) -> BTreeMap<Did, u64> {
    let mut nonces = BTreeMap::new();
    for (d, _) in signers {
        let did_detail = DIDModule::onchain_did_details(&d).unwrap();
        nonces.insert(*d, did_detail.nonce);
    }
    nonces
}

pub fn check_nonce_increase(old_nonces: BTreeMap<Did, u64>, signers: &[(Did, &sr25519::Pair)]) {
    let new_nonces = get_nonces(&signers);
    assert_eq!(new_nonces.len(), old_nonces.len());
    for (d, new_nonce) in new_nonces {
        assert_eq!(old_nonces.get(&d).unwrap() + 1, new_nonce);
    }
}

/// Tests every failure case in the module.
/// If a failure case is not covered, thats a bug.
/// If an error variant from TrustError is not covered, thats a bug.
///
/// Tests in this module are named after the errors they check.
/// For example, `#[test] fn invalidpolicy` exercises the TrustError::InvalidPolicy.
mod errors {
    // Cannot do `use super::*` as that would import `Call` as `Call` which conflicts with `Call` in
    // `test_common`
    use super::*;
    use alloc::collections::BTreeSet;
    use frame_support::dispatch::DispatchError;

    #[test]
    fn invalidpolicy() {
        if !in_ext() {
            return ext().execute_with(invalidpolicy);
        }

        let ar = AddAuthorizer {
            id: RGA,
            new_authorizer: Authorizer {
                policy: trusted_entity_oneof(&[]),
                add_only: false,
            },
        };

        let err = TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap_err();
        assert_eq!(err, TrustError::<Test>::InvalidPolicy.into());
    }

    // this test has caught at least one bug
    #[test]
    fn notauthorized() {
        if !in_ext() {
            return ext().execute_with(notauthorized);
        }

        fn assert_add_issuer_err(
            policy: Policy,
            signers: &[(Did, &sr25519::Pair)],
        ) -> DispatchError {
            let authorizer_id: AuthorizerId = random();
            let ar = AddAuthorizer {
                id: authorizer_id,
                new_authorizer: Authorizer {
                    policy,
                    add_only: false,
                },
            };
            TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

            let add_issuer = AddIssuerRaw {
                _marker: PhantomData,
                authorizer_id,
                entity_ids: random::<[TrustedEntityId; 32]>().iter().cloned().collect(),
            };
            let pauth = get_pauth(&add_issuer, signers);
            dbg!(&add_issuer);
            dbg!(&pauth);
            TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, pauth)
                .unwrap_err()
        }

        run_to_block(10);

        let (a, b, c) = (DIDA, DIDB, DIDC);
        let (kpa, kpb, kpc) = (create_did(a), create_did(b), create_did(c));

        let cases: &[(Policy, &[(Did, &sr25519::Pair)], &str)] = &[
            (trusted_entity_oneof(&[a]), &[], "provide no signatures"),
            (
                trusted_entity_oneof(&[a]),
                &[(b, &kpb)],
                "wrong account; wrong key",
            ),
            (
                trusted_entity_oneof(&[a]),
                &[(a, &kpb)],
                "correct account; wrong key",
            ),
            (
                trusted_entity_oneof(&[a]),
                &[(a, &kpb)],
                "wrong account; correct key",
            ),
            (
                trusted_entity_oneof(&[a, b]),
                &[(c, &kpc)],
                "account not a controller",
            ),
            (
                trusted_entity_oneof(&[a, b]),
                &[(a, &kpa), (b, &kpb)],
                "two signers",
            ),
            (trusted_entity_oneof(&[a]), &[], "one controller; no sigs"),
            (
                trusted_entity_oneof(&[a, b]),
                &[],
                "two controllers; no sigs",
            ),
        ];

        for (pol, set, description) in cases {
            dbg!(description);
            assert_eq!(
                assert_add_issuer_err(pol.clone(), set),
                TrustError::<Test>::NotAuthorized.into(),
                "{}",
                description
            );
        }
    }

    #[test]
    /// sign unrelated commands and ensure they fail
    fn notauthorized_wrong_command() {
        if !in_ext() {
            return ext().execute_with(notauthorized_wrong_command);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = false;

        run_to_block(10);

        let kpa = create_did(DIDA);
        let authorizer = Authorizer { policy, add_only };

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: authorizer,
        };
        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

        let remove_issuer = RemoveIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: once(Default::default()).collect(),
        };
        let ur_proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
        TrustedEntityMod::remove_issuer(
            RuntimeOrigin::signed(ABBA),
            remove_issuer.clone(),
            ur_proof,
        )
        .unwrap();

        let add_issuer = AddIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: once(Default::default()).collect(),
        };
        let ur_proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
        assert_eq!(
            TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, ur_proof)
                .unwrap_err(),
            TrustError::<Test>::NotAuthorized.into()
        );

        let ur_proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
        TrustedEntityMod::remove_issuer(
            RuntimeOrigin::signed(ABBA),
            remove_issuer.clone(),
            ur_proof,
        )
        .unwrap();
    }

    #[test]
    fn authorizer_exists() {
        if !in_ext() {
            return ext().execute_with(authorizer_exists);
        }

        let authorizer = Authorizer {
            policy: trusted_entity_oneof(&[DIDA]),
            add_only: false,
        };
        let ar = AddAuthorizer {
            id: RGA,
            new_authorizer: authorizer,
        };
        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar.clone()).unwrap();
        let err =
            TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar.clone()).unwrap_err();
        assert_eq!(err, TrustError::<Test>::AuthorizerExists.into());
    }

    #[test]
    fn no_authorizer() {
        if !in_ext() {
            return ext().execute_with(no_authorizer);
        }

        let authorizer_id = RGA;

        let no_authorizer: Result<(), DispatchError> = Err(TrustError::<Test>::NoAuthorizer.into());

        assert_eq!(
            TrustedEntityMod::add_issuer(
                RuntimeOrigin::signed(ABBA),
                AddIssuerRaw {
                    _marker: PhantomData,
                    authorizer_id,
                    entity_ids: once(Default::default()).collect(),
                },
                vec![]
            ),
            no_authorizer
        );
        assert_eq!(
            TrustedEntityMod::remove_issuer(
                RuntimeOrigin::signed(ABBA),
                RemoveIssuerRaw {
                    _marker: PhantomData,
                    authorizer_id,
                    entity_ids: once(Default::default()).collect(),
                },
                vec![],
            ),
            no_authorizer
        );
        assert_eq!(
            TrustedEntityMod::remove_authorizer(
                RuntimeOrigin::signed(ABBA),
                RemoveAuthorizerRaw {
                    _marker: PhantomData,
                    authorizer_id
                },
                vec![],
            ),
            no_authorizer
        );
    }

    #[test]
    fn too_many_controllers() {
        if !in_ext() {
            return ext().execute_with(incorrect_nonce);
        }

        let authorizer_id = RGA;
        let err = TrustError::<Test>::TooManyControllers;

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: Authorizer {
                policy: Policy::OneOf((0u8..16).map(U256::from).map(Into::into).map(Did).collect()),
                add_only: false,
            },
        };

        assert_noop!(
            TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar),
            err
        );
    }

    #[test]
    fn emtpy_payload() {
        if !in_ext() {
            return ext().execute_with(incorrect_nonce);
        }
        let err = TrustError::<Test>::EmptyPayload;

        let kpa = create_did(DIDA);
        let authorizer_id = RGA;
        let authorizer = Authorizer {
            policy: trusted_entity_oneof(&[DIDA]),
            add_only: false,
        };
        let ar = AddAuthorizer {
            id: RGA,
            new_authorizer: authorizer,
        };
        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar.clone()).unwrap();
        let add_issuer_raw = AddIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: Default::default(),
        };
        let proof = get_pauth(&add_issuer_raw, &[(DIDA, &kpa)]);

        assert_noop!(
            TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer_raw, proof),
            err
        );
    }

    #[test]
    fn incorrect_nonce() {
        if !in_ext() {
            return ext().execute_with(incorrect_nonce);
        }

        run_to_block(1);

        let kpa = create_did(DIDA);

        let authorizer_id = RGA;
        let err: Result<(), DispatchError> = Err(TrustError::<Test>::IncorrectNonce.into());

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: Authorizer {
                policy: trusted_entity_oneof(&[DIDA]),
                add_only: false,
            },
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

        let add_issuer = AddIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: once(Default::default()).collect(),
        };
        let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);

        // Increase nonce to make the auth chekc fail
        inc_nonce(&DIDA);
        assert_eq!(
            TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof),
            err
        );

        let remove_issuer = RemoveIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: once(Default::default()).collect(),
        };
        let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);

        // Increase nonce to make the auth check fail
        inc_nonce(&DIDA);
        assert_eq!(
            TrustedEntityMod::remove_issuer(RuntimeOrigin::signed(ABBA), remove_issuer, proof,),
            err
        );

        let remove = RemoveAuthorizerRaw {
            _marker: PhantomData,
            authorizer_id,
        };
        let proof = get_pauth(&remove, &[(DIDA, &kpa)]);

        // Increase nonce to make the auth check fail
        inc_nonce(&DIDA);
        assert_eq!(
            TrustedEntityMod::remove_authorizer(RuntimeOrigin::signed(ABBA), remove, proof,),
            err
        );
    }

    #[test]
    fn addonly() {
        if !in_ext() {
            return ext().execute_with(addonly);
        }

        let authorizer_id = RGA;
        let err: Result<(), DispatchError> = Err(TrustError::<Test>::AddOnly.into());
        let entity_ids: BTreeSet<_> = [RA, RB, RC].iter().cloned().collect();

        run_to_block(1);

        let kpa = create_did(DIDA);

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: Authorizer {
                policy: trusted_entity_oneof(&[DIDA]),
                add_only: true,
            },
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

        let remove_issuer = RemoveIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids,
        };
        let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
        assert_eq!(
            TrustedEntityMod::remove_issuer(RuntimeOrigin::signed(ABBA), remove_issuer, proof),
            err
        );

        let remove = RemoveAuthorizerRaw {
            _marker: PhantomData,
            authorizer_id,
        };
        let proof = get_pauth(&remove, &[(DIDA, &kpa)]);
        assert_eq!(
            TrustedEntityMod::remove_authorizer(RuntimeOrigin::signed(ABBA), remove, proof),
            err
        );
    }

    // Untested variants will be a match error.
    // To fix the match error, write a test for the variant then update the test.
    fn _all_included(dummy: TrustError<Test>) {
        match dummy {
            TrustError::__Ignore(_, _)
            | TrustError::InvalidPolicy
            | TrustError::NotAuthorized
            | TrustError::AuthorizerExists
            | TrustError::AuthorizerNotExists
            | TrustError::NoAuthorizer
            | TrustError::IncorrectNonce
            | TrustError::AddOnly
            | TrustError::EmptyPayload
            | TrustError::TooManyControllers => {}
        }
    }
}

/// Tests every happy path for every public extrinsic call in the module.
/// If a happy path is not covered, thats a bug.
/// If a call is not covered, thats a bug.
///
/// Tests in this module are named after the calls they check.
/// For example, `#[test] fn new_authorizer` tests the happy path for Module::new_authorizer.
mod calls {
    use super::*;
    // Cannot do `use super::super::*` as that would import `Call` as `Call` which conflicts with
    // `Call` in `test_common`
    use super::super::{Authorizers, Call as TrstCall, Issuers};
    use alloc::collections::BTreeSet;
    use frame_support::{StorageDoubleMap, StorageMap};

    #[test]
    fn new_authorizer() {
        if !in_ext() {
            return ext().execute_with(new_authorizer);
        }

        let cases: &[(Policy, bool)] = &[
            (trusted_entity_oneof(&[DIDA]), false),
            (trusted_entity_oneof(&[DIDA, DIDB]), false),
            (trusted_entity_oneof(&[DIDA]), true),
            (trusted_entity_oneof(&[DIDA, DIDB]), true),
        ];
        for (policy, add_only) in cases.iter().cloned() {
            let authorizer_id = random();
            let authorizer = Authorizer { policy, add_only };
            let ar = AddAuthorizer {
                id: authorizer_id,
                new_authorizer: authorizer.clone(),
            };
            assert!(!Authorizers::contains_key(&authorizer_id));
            TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
            assert!(Authorizers::contains_key(authorizer_id));
            assert_eq!(Authorizers::get(authorizer_id).unwrap(), authorizer);
        }
    }

    #[test]
    fn add_issuer() {
        if !in_ext() {
            return ext().execute_with(add_issuer);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = true;

        run_to_block(1);

        let kpa = create_did(DIDA);

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: Authorizer { policy, add_only },
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

        let cases: &[&[TrustedEntityId]] = &[
            // &[],
            &[random()],
            &[random(), random()],
            &[random(), random(), random()],
            &[RA], // Test idempotence, step 1
            &[RA], // Test idempotence, step 2
        ];
        for (i, ids) in cases.into_iter().enumerate() {
            println!("AddTrustedEntity ids: {:?}", ids);
            let add_issuer = AddIssuerRaw {
                _marker: PhantomData,
                authorizer_id,
                entity_ids: ids.iter().cloned().collect(),
            };
            let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);
            let old_nonces = get_nonces(&[((DIDA, &kpa))]);
            TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof).unwrap();
            assert!(ids
                .iter()
                .all(|id| Issuers::contains_key(authorizer_id, id)));
            check_nonce_increase(old_nonces, &[((DIDA, &kpa))]);
            run_to_block(1 + 1 + i as u64);
        }
    }

    #[test]
    fn remove_issuer() {
        if !in_ext() {
            return ext().execute_with(remove_issuer);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = false;

        run_to_block(10);

        let kpa = create_did(DIDA);

        enum Action {
            AddTrustedEntity,
            RemoveTrustedEntity,
            AddAuthorizer,
            RemoveAuthorizer,
        }

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: Authorizer { policy, add_only },
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

        let cases: &[(Action, &[TrustedEntityId], u32)] = &[
            //(Action::RemoveTrustedEntity, &[], line!()),
            (Action::RemoveTrustedEntity, &[random()], line!()),
            (Action::RemoveTrustedEntity, &[random(), random()], line!()),
            (
                Action::RemoveTrustedEntity,
                &[random(), random(), random()],
                line!(),
            ),
            (Action::AddTrustedEntity, &[RA, RB], line!()),
            (Action::AddAuthorizer, &[RA, RB], line!()),
            (Action::RemoveTrustedEntity, &[RA], line!()),
            (Action::RemoveAuthorizer, &[RA], line!()),
            (Action::AddAuthorizer, &[RB], line!()),
            (Action::RemoveTrustedEntity, &[RA, RB], line!()),
            (Action::RemoveAuthorizer, &[RA, RB], line!()),
            (Action::AddTrustedEntity, &[RA, RB], line!()),
            (Action::AddAuthorizer, &[RA, RB], line!()),
            (Action::RemoveTrustedEntity, &[RA, RB], line!()),
            (Action::RemoveAuthorizer, &[RA, RB], line!()),
        ];
        for (i, (action, ids, line_no)) in cases.into_iter().enumerate() {
            eprintln!("running action from line {}", line_no);
            let entity_ids: BTreeSet<TrustedEntityId> = ids.iter().cloned().collect();
            match action {
                Action::AddTrustedEntity => {
                    let add_issuer = AddIssuerRaw {
                        _marker: PhantomData,
                        authorizer_id,
                        entity_ids,
                    };
                    let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);
                    let old_nonces = get_nonces(&[((DIDA, &kpa))]);
                    TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof)
                        .unwrap();
                    check_nonce_increase(old_nonces, &[((DIDA, &kpa))]);
                }
                Action::RemoveTrustedEntity => {
                    let remove_issuer = RemoveIssuerRaw {
                        _marker: PhantomData,
                        authorizer_id,
                        entity_ids: entity_ids.clone(),
                    };
                    let old_nonces = get_nonces(&[((DIDA, &kpa))]);
                    let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
                    TrustedEntityMod::remove_issuer(
                        RuntimeOrigin::signed(ABBA),
                        remove_issuer,
                        proof,
                    )
                    .unwrap();
                    check_nonce_increase(old_nonces, &[((DIDA, &kpa))]);
                }
                Action::AddAuthorizer => {
                    assert!(entity_ids
                        .iter()
                        .all(|id| Issuers::contains_key(authorizer_id, id)));
                }
                Action::RemoveAuthorizer => {
                    assert!(!entity_ids
                        .iter()
                        .any(|id| Issuers::contains_key(authorizer_id, id)));
                }
            }
            run_to_block(10 + 1 + i as u64)
        }
    }

    #[test]
    fn remove_authorizer() {
        if !in_ext() {
            return ext().execute_with(remove_authorizer);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = false;
        let kpa = create_did(DIDA);

        let authorizer = Authorizer { policy, add_only };
        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: authorizer.clone(),
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
        assert!(Authorizers::contains_key(authorizer_id));

        // destroy authorizer
        let rem = RemoveAuthorizerRaw {
            _marker: PhantomData,
            authorizer_id,
        };
        let proof = get_pauth(&rem, &[(DIDA, &kpa)]);
        let old_nonces = get_nonces(&[((DIDA, &kpa))]);
        TrustedEntityMod::remove_authorizer(RuntimeOrigin::signed(ABBA), rem, proof).unwrap();
        check_nonce_increase(old_nonces, &[((DIDA, &kpa))]);

        // assert not exists
        assert!(!Authorizers::contains_key(authorizer_id));
    }

    // Untested variants will be a match error.
    // To fix the match error, write a test for the variant then update the test.
    fn _all_included(dummy: TrstCall<Test>) {
        match dummy {
            TrstCall::new_authorizer { .. }
            | TrstCall::add_policy_controller { .. }
            | TrstCall::remove_policy_controller { .. }
            | TrstCall::add_issuer { .. }
            | TrstCall::remove_issuer { .. }
            | TrstCall::add_verifier { .. }
            | TrstCall::remove_verifier { .. }
            | TrstCall::remove_authorizer { .. }
            | TrstCall::__PhantomItem { .. } => {}
        }
    }
}

mod test {
    use frame_support::StorageMap;
    use sp_runtime::DispatchError;
    // Cannot do `use super::*` as that would import `Call` as `Call` which conflicts with `Call` in
    // `test_common`
    use super::*;
    use crate::trusted_entity::Authorizers;

    #[test]
    /// Exercises Module::ensure_auth, both success and failure cases.
    fn ensure_auth() {
        if !in_ext() {
            return ext().execute_with(ensure_auth);
        }

        run_to_block(10);

        let (a, b, c): (Did, Did, Did) = (Did(random()), Did(random()), Did(random()));
        let (kpa, kpb, kpc) = (create_did(a), create_did(b), create_did(c));
        let add_issuer = AddIssuerRaw {
            _marker: PhantomData,
            authorizer_id: RGA,
            entity_ids: once(Default::default()).collect(),
        };

        let cases: &[(u32, Policy, &[(Did, &sr25519::Pair)], bool)] = &[
            (line!(), trusted_entity_oneof(&[a]), &[(a, &kpa)], true),
            (line!(), trusted_entity_oneof(&[a, b]), &[(a, &kpa)], true),
            (line!(), trusted_entity_oneof(&[a, b]), &[(b, &kpb)], true),
            (line!(), trusted_entity_oneof(&[a]), &[], false), // provide no signatures
            (line!(), trusted_entity_oneof(&[a]), &[(b, &kpb)], false), // wrong account; wrong key
            (line!(), trusted_entity_oneof(&[a]), &[(a, &kpb)], false), // correct account; wrong key
            (line!(), trusted_entity_oneof(&[a]), &[(a, &kpb)], false), // wrong account; correct key
            (line!(), trusted_entity_oneof(&[a, b]), &[(c, &kpc)], false), // account not a controller
            (
                line!(),
                trusted_entity_oneof(&[a, b]),
                &[(a, &kpa), (b, &kpb)],
                false,
            ), // two signers
            (line!(), trusted_entity_oneof(&[a]), &[], false), // one controller; no sigs
            (line!(), trusted_entity_oneof(&[a, b]), &[], false), // two controllers; no sigs
        ];
        for (i, (line_no, policy, signers, expect_success)) in cases.into_iter().enumerate() {
            eprintln!("running case from line {}", line_no);
            Authorizers::insert(
                RGA,
                Authorizer {
                    policy: policy.clone(),
                    add_only: false,
                },
            );

            let old_nonces = get_nonces(signers);
            let command = &add_issuer;
            let proof = get_pauth(command, &signers);
            let res = TrustedEntityMod::try_exec_action_over_authorizer(
                command.clone(),
                proof,
                |_, _| Ok::<_, DispatchError>(()),
            );
            assert_eq!(res.is_ok(), *expect_success);

            if *expect_success {
                check_nonce_increase(old_nonces, signers);
            }
            run_to_block(10 + 1 + i as u64);
        }
    }

    #[test]
    /// Exercises the trusted entity convenience getter, get_authorizer.
    fn get_authorizer() {
        if !in_ext() {
            return ext().execute_with(get_authorizer);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = false;
        let authorizer = Authorizer { policy, add_only };

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: authorizer.clone(),
        };

        assert_eq!(TrustedEntityMod::get_authorizer(authorizer_id), None);
        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
        assert_eq!(
            TrustedEntityMod::get_authorizer(authorizer_id),
            Some(authorizer)
        );
    }

    #[test]
    /// Exercises the trusted entity status convenience getter, get_issuer.
    fn get_issuer() {
        if !in_ext() {
            return ext().execute_with(get_issuer);
        }

        let policy = trusted_entity_oneof(&[DIDA]);
        let authorizer_id = RGA;
        let add_only = false;
        let authorizer = Authorizer { policy, add_only };
        let kpa = create_did(DIDA);
        let revid: TrustedEntityId = random();

        let ar = AddAuthorizer {
            id: authorizer_id,
            new_authorizer: authorizer.clone(),
        };

        TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
        let add_issuer = AddIssuerRaw {
            _marker: PhantomData,
            authorizer_id,
            entity_ids: once(revid).collect(),
        };
        let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);

        assert_eq!(TrustedEntityMod::get_issuer(authorizer_id, revid), None);
        TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof).unwrap();
        assert_eq!(TrustedEntityMod::get_issuer(authorizer_id, revid), Some(()));
    }
}
