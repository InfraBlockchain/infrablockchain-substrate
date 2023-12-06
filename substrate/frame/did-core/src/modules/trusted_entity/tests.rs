#![allow(clippy::type_complexity)]

use super::*;
use crate::{
	common::{Policy, ToStateChange},
	did::Did,
	tests::common::*,
	util::{Action, WithNonce},
};
use alloc::collections::BTreeMap;
use frame_support::assert_noop;
use frame_system::Origin;
use sp_core::{sr25519, U256};
use sp_runtime::DispatchError;
use sp_std::{iter::once, marker::PhantomData};

pub fn get_pauth<A: Action + Clone>(
	action: &A,
	signers: &[(Did, &sr25519::Pair)],
) -> Vec<DidSignatureWithNonce<Test>>
where
	WithNonce<Test, A>: ToStateChange<Test>,
{
	signers
		.iter()
		.map(|(did, kp)| {
			let did_detail = DIDModule::onchain_did_details(did).unwrap();
			let next_nonce = did_detail.next_nonce().unwrap();
			let sp = WithNonce::<Test, _>::new_with_nonce(action.clone(), next_nonce);
			let sig = did_sig_on_bytes(&sp.to_state_change().encode(), kp, *did, 1);
			DidSignatureWithNonce { sig, nonce: next_nonce }
		})
		.collect()
}

pub fn get_nonces(signers: &[(Did, &sr25519::Pair)]) -> BTreeMap<Did, u64> {
	let mut nonces = BTreeMap::new();
	for (d, _) in signers {
		let did_detail = DIDModule::onchain_did_details(d).unwrap();
		nonces.insert(*d, did_detail.nonce);
	}
	nonces
}

pub fn check_nonce_increase(old_nonces: BTreeMap<Did, u64>, signers: &[(Did, &sr25519::Pair)]) {
	let new_nonces = get_nonces(signers);
	assert_eq!(new_nonces.len(), old_nonces.len());
	for (d, new_nonce) in new_nonces {
		assert_eq!(old_nonces.get(&d).unwrap() + 1, new_nonce);
	}
}

/// Tests every failure case in the module.
/// If a failure case is not covered, thats a bug.
/// If an error variant from Error is not covered, thats a bug.
///
/// Tests in this module are named after the errors they check.
/// For example, `#[test] fn invalidpolicy` exercises the Error::InvalidPolicy.
mod errors {
	use crate::common::{PolicyExecutionError, PolicyValidationError};

	// Cannot do `use super::*` as that would import `Call` as `Call` which conflicts with `Call` in
	// `tests::common`
	use super::*;
	use alloc::collections::BTreeSet;

	#[test]
	fn invalidpolicy() {
		if !in_ext() {
			return ext().execute_with(invalidpolicy)
		}

		let ar = AddAuthorizer {
			id: AUA,
			new_authorizer: Authorizer {
				policy: Policy::one_of(None::<Did>).unwrap(),
				add_only: false,
			},
		};

		let err = TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap_err();
		assert_eq!(err, PolicyValidationError::Empty.into());
	}

	// this test has caught at least one bug
	#[test]
	fn notauthorized() {
		if !in_ext() {
			return ext().execute_with(notauthorized)
		}

		fn assert_add_issuer_err(
			policy: Policy<Test>,
			signers: &[(Did, &sr25519::Pair)],
		) -> DispatchError {
			let authorizer_id: AuthorizerId = AuthorizerId(random());
			let ar = AddAuthorizer {
				id: authorizer_id,
				new_authorizer: Authorizer { policy, add_only: false },
			};
			TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

			let add_issuer_raw = AddIssuerRaw {
				_marker: PhantomData,
				authorizer_id,
				entity_ids: random::<[[u8; 32]; 32]>().iter().cloned().map(Into::into).collect(),
			};
			let pauth = get_pauth(&add_issuer_raw, signers);
			dbg!(&add_issuer_raw);
			dbg!(&pauth);
			TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer_raw, pauth)
				.unwrap_err()
		}

		run_to_block(10);

		let (a, b, c) = (DIDA, DIDB, DIDC);
		let (kpa, kpb, kpc) = (create_did(a), create_did(b), create_did(c));

		let cases: &[(Policy<Test>, &[(Did, &sr25519::Pair)], &str)] = &[
			(Policy::one_of([a]).unwrap(), &[], "provide no signatures"),
			(Policy::one_of([a]).unwrap(), &[(b, &kpb)], "wrong account; wrong key"),
			(Policy::one_of([a]).unwrap(), &[(a, &kpb)], "correct account; wrong key"),
			(Policy::one_of([a]).unwrap(), &[(a, &kpb)], "wrong account; correct key"),
			(Policy::one_of([a, b]).unwrap(), &[(c, &kpc)], "account not a controller"),
			(Policy::one_of([a, b]).unwrap(), &[(a, &kpa), (b, &kpb)], "two signers"),
			(Policy::one_of([a]).unwrap(), &[], "one controller; no sigs"),
			(Policy::one_of([a, b]).unwrap(), &[], "two controllers; no sigs"),
		];

		for (pol, set, description) in cases {
			dbg!(description);
			assert_eq!(
				assert_add_issuer_err(pol.clone(), set),
				PolicyExecutionError::NotAuthorized.into(),
				"{}",
				description
			);
		}
	}

	#[test]
	/// sign unrelated commands and ensure they fail
	fn notauthorized_wrong_command() {
		if !in_ext() {
			return ext().execute_with(notauthorized_wrong_command)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;

		run_to_block(10);

		let kpa = create_did(DIDA);
		let _kpb = create_did(DIDB);
		let authorizer = Authorizer { policy, add_only };

		let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer };
		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let remove_issuer_raw = RemoveIssuerRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(TrustedEntityId(Default::default())).collect(),
		};
		let ur_proof = get_pauth(&remove_issuer_raw, &[(DIDA, &kpa)]);
		TrustedEntityMod::remove_issuer(
			RuntimeOrigin::signed(ABBA),
			remove_issuer_raw.clone(),
			ur_proof,
		)
		.unwrap();

		let add_issuer_raw = AddIssuerRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(TrustedEntityId(Default::default())).collect(),
		};
		let ur_proof = get_pauth(&add_issuer_raw, &[(DIDB, &kpa)]);
		assert_eq!(
			TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer_raw, ur_proof)
				.unwrap_err(),
			PolicyExecutionError::NotAuthorized.into()
		);

		let ur_proof = get_pauth(&remove_issuer_raw, &[(DIDA, &kpa)]);
		TrustedEntityMod::remove_issuer(RuntimeOrigin::signed(ABBA), remove_issuer_raw, ur_proof)
			.unwrap();
	}

	#[test]
	fn authzexists() {
		if !in_ext() {
			return ext().execute_with(authzexists)
		}

		let authorizer = Authorizer { policy: Policy::one_of([DIDA]).unwrap(), add_only: false };
		let ar = AddAuthorizer { id: AUA, new_authorizer: authorizer };
		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar.clone()).unwrap();
		let err = TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap_err();
		assert_eq!(err, Error::<Test>::AuthzExists.into());
	}

	#[test]
	fn noreg() {
		if !in_ext() {
			return ext().execute_with(noreg)
		}

		let authorizer_id = AUA;

		let noreg: Result<(), DispatchError> = Err(PolicyExecutionError::NoEntity.into());

		assert_eq!(
			TrustedEntityMod::add_issuer(
				RuntimeOrigin::signed(ABBA),
				AddIssuerRaw {
					_marker: PhantomData,
					authorizer_id,
					entity_ids: once(TrustedEntityId(Default::default())).collect(),
				},
				vec![]
			),
			noreg
		);
		assert_eq!(
			TrustedEntityMod::remove_issuer(
				RuntimeOrigin::signed(ABBA),
				RemoveIssuerRaw {
					_marker: PhantomData,
					authorizer_id,
					entity_ids: once(TrustedEntityId(Default::default())).collect(),
				},
				vec![],
			),
			noreg
		);
		assert_eq!(
			TrustedEntityMod::remove_authorizer(
				RuntimeOrigin::signed(ABBA),
				RemoveAuthorizerRaw { _marker: PhantomData, authorizer_id },
				vec![],
			),
			noreg
		);
	}

	#[test]
	fn too_many_controllers() {
		if !in_ext() {
			return ext().execute_with(incorrect_nonce)
		}

		let authorizer_id = AUA;
		let err = Error::<Test>::TooManyControllers;

		let ar = AddAuthorizer {
			id: authorizer_id,
			new_authorizer: Authorizer {
				policy: Policy::one_of((0u8..16).map(U256::from).map(Into::into).map(Did)).unwrap(),
				add_only: false,
			},
		};

		assert_noop!(TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar), err);
	}

	#[test]
	fn emtpy_payload() {
		if !in_ext() {
			return ext().execute_with(incorrect_nonce)
		}
		let err = Error::<Test>::EmptyPayload;

		let kpa = create_did(DIDA);
		let authorizer_id = AUA;
		let authorizer = Authorizer { policy: Policy::one_of([DIDA]).unwrap(), add_only: false };
		let ar = AddAuthorizer { id: AUA, new_authorizer: authorizer };
		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
		let add_issuer_raw =
			AddIssuerRaw { _marker: PhantomData, authorizer_id, entity_ids: Default::default() };
		let proof = get_pauth(&add_issuer_raw, &[(DIDA, &kpa)]);

		assert_noop!(
			TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer_raw, proof),
			err
		);
	}

	#[test]
	fn incorrect_nonce() {
		if !in_ext() {
			return ext().execute_with(incorrect_nonce)
		}

		run_to_block(1);

		let kpa = create_did(DIDA);

		let authorizer_id = AUA;
		let err: Result<(), DispatchError> = Err(PolicyExecutionError::IncorrectNonce.into());

		let ar = AddAuthorizer {
			id: authorizer_id,
			new_authorizer: Authorizer { policy: Policy::one_of([DIDA]).unwrap(), add_only: false },
		};

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let add_issuer_raw = AddIssuerRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(TrustedEntityId(Default::default())).collect(),
		};
		let proof = get_pauth(&add_issuer_raw, &[(DIDA, &kpa)]);

		// Increase nonce to make the auth chekc fail
		inc_nonce(&DIDA);
		assert_eq!(
			TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer_raw, proof),
			err
		);

		let remove_issuer = RemoveIssuerRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(TrustedEntityId(Default::default())).collect(),
		};
		let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);

		// Increase nonce to make the auth check fail
		inc_nonce(&DIDA);
		assert_eq!(
			TrustedEntityMod::remove_issuer(RuntimeOrigin::signed(ABBA), remove_issuer, proof,),
			err
		);

		let remove = RemoveAuthorizerRaw { _marker: PhantomData, authorizer_id };
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
			return ext().execute_with(addonly)
		}

		let authorizer_id = AUA;
		let err: Result<(), DispatchError> = Err(Error::<Test>::AddOnly.into());
		let entity_ids: BTreeSet<_> = [TEA, TEB, TEC].iter().cloned().collect();

		run_to_block(1);

		let kpa = create_did(DIDA);

		let ar = AddAuthorizer {
			id: authorizer_id,
			new_authorizer: Authorizer { policy: Policy::one_of([DIDA]).unwrap(), add_only: true },
		};

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let remove_issuer = RemoveIssuerRaw { _marker: PhantomData, authorizer_id, entity_ids };
		let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
		assert_eq!(
			TrustedEntityMod::remove_issuer(RuntimeOrigin::signed(ABBA), remove_issuer, proof),
			err
		);

		let remove = RemoveAuthorizerRaw { _marker: PhantomData, authorizer_id };
		let proof = get_pauth(&remove, &[(DIDA, &kpa)]);
		assert_eq!(
			TrustedEntityMod::remove_authorizer(RuntimeOrigin::signed(ABBA), remove, proof),
			err
		);
	}

	// Untested variants will be a match error.
	// To fix the match error, write a test for the variant then update the test.
	fn _all_included(dummy: Error<Test>) {
		match dummy {
			Error::__Ignore(_, _) |
			Error::AuthzExists |
			Error::EmptyPayload |
			Error::IncorrectNonce |
			Error::AddOnly |
			Error::TooManyControllers => {},
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
	// `Call` in `tests::common`
	use super::super::{Authorizers, Call as RevCall, Issuers, Verifiers};
	use alloc::collections::BTreeSet;

	#[test]
	fn new_authorizer() {
		if !in_ext() {
			return ext().execute_with(new_authorizer)
		}

		let cases: &[(Policy<Test>, bool)] = &[
			(Policy::one_of([DIDA]).unwrap(), false),
			(Policy::one_of([DIDA, DIDB]).unwrap(), false),
			(Policy::one_of([DIDA]).unwrap(), true),
			(Policy::one_of([DIDA, DIDB]).unwrap(), true),
		];
		for (policy, add_only) in cases.iter().cloned() {
			let authorizer_id = AuthorizerId(random());
			let authorizer = Authorizer { policy, add_only };
			let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer.clone() };
			assert!(!Authorizers::<Test>::contains_key(authorizer_id));
			TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
			assert!(Authorizers::<Test>::contains_key(authorizer_id));
			assert_eq!(Authorizers::<Test>::get(authorizer_id).unwrap(), authorizer);
		}
	}

	#[test]
	fn add_issuer() {
		if !in_ext() {
			return ext().execute_with(add_issuer)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = true;

		run_to_block(1);

		let kpa = create_did(DIDA);

		let ar =
			AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy, add_only } };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let cases: &[&[TrustedEntityId]] = &[
			// &[],
			&[TrustedEntityId(random())],
			&[TrustedEntityId(random()), TrustedEntityId(random())],
			&[TrustedEntityId(random()), TrustedEntityId(random()), TrustedEntityId(random())],
			&[TEA], // Test idempotence, step 1
			&[TEA], // Test idempotence, step 2
		];
		for (i, ids) in cases.iter().enumerate() {
			println!("AddIssuer ids: {:?}", ids);
			let add_issuer = AddIssuerRaw {
				_marker: PhantomData,
				authorizer_id,
				entity_ids: ids.iter().cloned().collect(),
			};
			let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);
			let old_nonces = get_nonces(&[(DIDA, &kpa)]);
			TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof).unwrap();
			assert!(ids.iter().all(|id| Issuers::<Test>::contains_key(authorizer_id, id)));
			check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
			run_to_block(1 + 1 + i as u64);
		}
	}

	#[test]
	fn remove_issuer() {
		if !in_ext() {
			return ext().execute_with(remove_issuer)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;

		run_to_block(10);

		let kpa = create_did(DIDA);

		enum Action {
			AddIssuer,
			RemoveIssuer,
			AsrtIssuer,    // assert issuer
			AsrtNotIssuer, // assert not issuer
		}

		let ar =
			AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy, add_only } };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let cases: &[(Action, &[TrustedEntityId], u32)] = &[
			//(Action::RemoveIssuer, &[], line!()),
			(Action::RemoveIssuer, &[TrustedEntityId(random())], line!()),
			(
				Action::RemoveIssuer,
				&[TrustedEntityId(random()), TrustedEntityId(random())],
				line!(),
			),
			(
				Action::RemoveIssuer,
				&[TrustedEntityId(random()), TrustedEntityId(random()), TrustedEntityId(random())],
				line!(),
			),
			(Action::AddIssuer, &[TEA, TEB], line!()),
			(Action::AsrtIssuer, &[TEA, TEB], line!()),
			(Action::RemoveIssuer, &[TEA], line!()),
			(Action::AsrtNotIssuer, &[TEA], line!()),
			(Action::AsrtIssuer, &[TEB], line!()),
			(Action::RemoveIssuer, &[TEA, TEB], line!()),
			(Action::AsrtNotIssuer, &[TEA, TEB], line!()),
			(Action::AddIssuer, &[TEA, TEB], line!()),
			(Action::AsrtIssuer, &[TEA, TEB], line!()),
			(Action::RemoveIssuer, &[TEA, TEB], line!()),
			(Action::AsrtNotIssuer, &[TEA, TEB], line!()),
		];
		for (i, (action, ids, line_no)) in cases.iter().enumerate() {
			eprintln!("running action from line {}", line_no);
			let entity_ids: BTreeSet<TrustedEntityId> = ids.iter().cloned().collect();
			match action {
				Action::AddIssuer => {
					let add_issuer =
						AddIssuerRaw { _marker: PhantomData, authorizer_id, entity_ids };
					let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);
					let old_nonces = get_nonces(&[(DIDA, &kpa)]);
					TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof)
						.unwrap();
					check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
				},
				Action::RemoveIssuer => {
					let remove_issuer = RemoveIssuerRaw {
						_marker: PhantomData,
						authorizer_id,
						entity_ids: entity_ids.clone(),
					};
					let old_nonces = get_nonces(&[(DIDA, &kpa)]);
					let proof = get_pauth(&remove_issuer, &[(DIDA, &kpa)]);
					TrustedEntityMod::remove_issuer(
						RuntimeOrigin::signed(ABBA),
						remove_issuer,
						proof,
					)
					.unwrap();
					check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
				},
				Action::AsrtIssuer => {
					assert!(entity_ids
						.iter()
						.all(|id| Issuers::<Test>::contains_key(authorizer_id, id)));
				},
				Action::AsrtNotIssuer => {
					assert!(!entity_ids
						.iter()
						.any(|id| Issuers::<Test>::contains_key(authorizer_id, id)));
				},
			}
			run_to_block(10 + 1 + i as u64)
		}
	}

	#[test]
	fn add_verifier() {
		if !in_ext() {
			return ext().execute_with(add_verifier)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = true;

		run_to_block(1);

		let kpa = create_did(DIDA);

		let ar =
			AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy, add_only } };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let cases: &[&[TrustedEntityId]] = &[
			// &[],
			&[TrustedEntityId(random())],
			&[TrustedEntityId(random()), TrustedEntityId(random())],
			&[TrustedEntityId(random()), TrustedEntityId(random()), TrustedEntityId(random())],
			&[TEA], // Test idempotence, step 1
			&[TEA], // Test idempotence, step 2
		];
		for (i, ids) in cases.iter().enumerate() {
			println!("AddVerifier ids: {:?}", ids);
			let add_verifier_raw = AddVerifierRaw {
				_marker: PhantomData,
				authorizer_id,
				entity_ids: ids.iter().cloned().collect(),
			};
			let proof = get_pauth(&add_verifier_raw, &[(DIDA, &kpa)]);
			let old_nonces = get_nonces(&[(DIDA, &kpa)]);
			TrustedEntityMod::add_verifier(RuntimeOrigin::signed(ABBA), add_verifier_raw, proof)
				.unwrap();
			assert!(ids.iter().all(|id| Verifiers::<Test>::contains_key(authorizer_id, id)));
			check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
			run_to_block(1 + 1 + i as u64);
		}
	}

	#[test]
	fn remove_verifier() {
		if !in_ext() {
			return ext().execute_with(remove_verifier)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;

		run_to_block(10);

		let kpa = create_did(DIDA);

		enum Action {
			AddVerifier,
			RemoveVerifier,
			AsrtVerifier,    // assert verifier
			AsrtNotVerifier, // assert not verifier
		}

		let ar =
			AddAuthorizer { id: authorizer_id, new_authorizer: Authorizer { policy, add_only } };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();

		let cases: &[(Action, &[TrustedEntityId], u32)] = &[
			//(Action::RemoveVerifier, &[], line!()),
			(Action::RemoveVerifier, &[TrustedEntityId(random())], line!()),
			(
				Action::RemoveVerifier,
				&[TrustedEntityId(random()), TrustedEntityId(random())],
				line!(),
			),
			(
				Action::RemoveVerifier,
				&[TrustedEntityId(random()), TrustedEntityId(random()), TrustedEntityId(random())],
				line!(),
			),
			(Action::AddVerifier, &[TEA, TEB], line!()),
			(Action::AsrtVerifier, &[TEA, TEB], line!()),
			(Action::RemoveVerifier, &[TEA], line!()),
			(Action::AsrtNotVerifier, &[TEA], line!()),
			(Action::AsrtVerifier, &[TEB], line!()),
			(Action::RemoveVerifier, &[TEA, TEB], line!()),
			(Action::AsrtNotVerifier, &[TEA, TEB], line!()),
			(Action::AddVerifier, &[TEA, TEB], line!()),
			(Action::AsrtVerifier, &[TEA, TEB], line!()),
			(Action::RemoveVerifier, &[TEA, TEB], line!()),
			(Action::AsrtNotVerifier, &[TEA, TEB], line!()),
		];
		for (i, (action, ids, line_no)) in cases.iter().enumerate() {
			eprintln!("running action from line {}", line_no);
			let entity_ids: BTreeSet<TrustedEntityId> = ids.iter().cloned().collect();
			match action {
				Action::AddVerifier => {
					let add_verifier_raw =
						AddVerifierRaw { _marker: PhantomData, authorizer_id, entity_ids };
					let proof = get_pauth(&add_verifier_raw, &[(DIDA, &kpa)]);
					let old_nonces = get_nonces(&[(DIDA, &kpa)]);
					TrustedEntityMod::add_verifier(
						RuntimeOrigin::signed(ABBA),
						add_verifier_raw,
						proof,
					)
					.unwrap();
					check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
				},
				Action::RemoveVerifier => {
					let remove_verifier_raw = RemoveVerifierRaw {
						_marker: PhantomData,
						authorizer_id,
						entity_ids: entity_ids.clone(),
					};
					let old_nonces = get_nonces(&[(DIDA, &kpa)]);
					let proof = get_pauth(&remove_verifier_raw, &[(DIDA, &kpa)]);
					TrustedEntityMod::remove_verifier(
						RuntimeOrigin::signed(ABBA),
						remove_verifier_raw,
						proof,
					)
					.unwrap();
					check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);
				},
				Action::AsrtVerifier => {
					assert!(entity_ids
						.iter()
						.all(|id| Verifiers::<Test>::contains_key(authorizer_id, id)));
				},
				Action::AsrtNotVerifier => {
					assert!(!entity_ids
						.iter()
						.any(|id| Verifiers::<Test>::contains_key(authorizer_id, id)));
				},
			}
			run_to_block(10 + 1 + i as u64)
		}
	}

	#[test]
	fn remove_authorizer() {
		if !in_ext() {
			return ext().execute_with(remove_authorizer)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;
		let kpa = create_did(DIDA);

		let authorizer = Authorizer { policy, add_only };
		let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
		assert!(Authorizers::<Test>::contains_key(authorizer_id));

		// destroy authorizer
		let rem = RemoveAuthorizerRaw { _marker: PhantomData, authorizer_id };
		let proof = get_pauth(&rem, &[(DIDA, &kpa)]);
		let old_nonces = get_nonces(&[(DIDA, &kpa)]);
		TrustedEntityMod::remove_authorizer(RuntimeOrigin::signed(ABBA), rem, proof).unwrap();
		check_nonce_increase(old_nonces, &[(DIDA, &kpa)]);

		// assert not exists
		assert!(!Authorizers::<Test>::contains_key(authorizer_id));
	}

	// Untested variants will be a match error.
	// To fix the match error, write a test for the variant then update the test.
	fn _all_included(dummy: RevCall<Test>) {
		match dummy {
			RevCall::new_authorizer { .. } |
			RevCall::add_issuer { .. } |
			RevCall::remove_issuer { .. } |
			RevCall::add_verifier { .. } |
			RevCall::remove_verifier { .. } |
			RevCall::remove_authorizer { .. } |
			RevCall::__Ignore { .. } => {},
		}
	}
}

mod test {
	use sp_runtime::DispatchError;
	// Cannot do `use super::*` as that would import `Call` as `Call` which conflicts with `Call` in
	// `tests::common`
	use super::*;
	use crate::trusted_entity::Authorizers;

	#[test]
	/// Exercises Module::ensure_auth, both success and failure cases.
	fn ensure_auth() {
		if !in_ext() {
			return ext().execute_with(ensure_auth)
		}

		run_to_block(10);

		let (a, b, c): (Did, Did, Did) = (Did(random()), Did(random()), Did(random()));
		let (kpa, kpb, kpc) = (create_did(a), create_did(b), create_did(c));
		let add_issuer_raw = AddIssuerRaw {
			_marker: PhantomData,
			authorizer_id: AUA,
			entity_ids: once(TrustedEntityId(Default::default())).collect(),
		};

		let cases: &[(u32, Policy<Test>, &[(Did, &sr25519::Pair)], bool)] = &[
			(line!(), Policy::one_of([a]).unwrap(), &[(a, &kpa)], true),
			(line!(), Policy::one_of([a, b]).unwrap(), &[(a, &kpa)], true),
			(line!(), Policy::one_of([a, b]).unwrap(), &[(b, &kpb)], true),
			(line!(), Policy::one_of([a]).unwrap(), &[], false), // provide no signatures
			(line!(), Policy::one_of([a]).unwrap(), &[(b, &kpb)], false), // wrong account; wrong key
			(line!(), Policy::one_of([a]).unwrap(), &[(a, &kpb)], false), // correct account; wrong key
			(line!(), Policy::one_of([a]).unwrap(), &[(a, &kpb)], false), // wrong account; correct key
			(line!(), Policy::one_of([a, b]).unwrap(), &[(c, &kpc)], false), // account not a controller
			(line!(), Policy::one_of([a, b]).unwrap(), &[(a, &kpa), (b, &kpb)], false), // two signers
			(line!(), Policy::one_of([a]).unwrap(), &[], false), // one controller; no sigs
			(line!(), Policy::one_of([a, b]).unwrap(), &[], false), // two controllers; no sigs
		];
		for (i, (line_no, policy, signers, expect_success)) in cases.iter().enumerate() {
			eprintln!("running case from line {}", line_no);
			Authorizers::<Test>::insert(
				AUA,
				Authorizer { policy: policy.clone(), add_only: false },
			);

			let old_nonces = get_nonces(signers);
			let command = &add_issuer_raw;
			let proof = get_pauth(command, signers);
			let res = TrustedEntityMod::try_exec_action_over_authorizer(
				|_, _| Ok::<_, DispatchError>(()),
				command.clone(),
				proof,
			);
			assert_eq!(res.is_ok(), *expect_success);

			if *expect_success {
				check_nonce_increase(old_nonces, signers);
			}
			run_to_block(10 + 1 + i as u64);
		}
	}

	#[test]
	/// Exercises the revocation authorizer convenience getter, get_authorizer.
	fn get_authorizer() {
		if !in_ext() {
			return ext().execute_with(get_authorizer)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;
		let authorizer = Authorizer { policy, add_only };

		let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer.clone() };

		assert_eq!(TrustedEntityMod::get_authorizer(authorizer_id), None);
		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
		assert_eq!(TrustedEntityMod::get_authorizer(authorizer_id), Some(authorizer));
	}

	#[test]
	/// Exercises the revocation status convenience getter, get_issuer.
	fn get_issuer() {
		if !in_ext() {
			return ext().execute_with(get_issuer)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;
		let authorizer = Authorizer { policy, add_only };
		let kpa = create_did(DIDA);
		let entity_id: TrustedEntityId = TrustedEntityId(random());

		let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
		let add_issuer = AddIssuerRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(entity_id).collect(),
		};
		let proof = get_pauth(&add_issuer, &[(DIDA, &kpa)]);

		assert_eq!(TrustedEntityMod::get_issuer(authorizer_id, entity_id), None);
		TrustedEntityMod::add_issuer(RuntimeOrigin::signed(ABBA), add_issuer, proof).unwrap();
		assert_eq!(TrustedEntityMod::get_issuer(authorizer_id, entity_id), Some(()));
	}

	#[test]
	/// Exercises the revocation status convenience getter, get_verifier.
	fn get_verifier() {
		if !in_ext() {
			return ext().execute_with(get_verifier)
		}

		let policy = Policy::one_of([DIDA]).unwrap();
		let authorizer_id = AUA;
		let add_only = false;
		let authorizer = Authorizer { policy, add_only };
		let kpa = create_did(DIDA);
		let entity_id: TrustedEntityId = TrustedEntityId(random());

		let ar = AddAuthorizer { id: authorizer_id, new_authorizer: authorizer };

		TrustedEntityMod::new_authorizer(RuntimeOrigin::signed(ABBA), ar).unwrap();
		let add_verifier_raw = AddVerifierRaw {
			_marker: PhantomData,
			authorizer_id,
			entity_ids: once(entity_id).collect(),
		};
		let proof = get_pauth(&add_verifier_raw, &[(DIDA, &kpa)]);

		assert_eq!(TrustedEntityMod::get_verifier(authorizer_id, entity_id), None);
		TrustedEntityMod::add_verifier(RuntimeOrigin::signed(ABBA), add_verifier_raw, proof)
			.unwrap();
		assert_eq!(TrustedEntityMod::get_verifier(authorizer_id, entity_id), Some(()));
	}
}
