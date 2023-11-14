use super::*;
use crate::deposit_indexed_event;

impl<T: Config + Debug> Module<T> {
    pub(super) fn new_authorizer_(
        AddAuthorizer { new_authorizer, id }: AddAuthorizer,
    ) -> DispatchResult {
        // check
        ensure!(
            new_authorizer.policy.valid(),
            TrustError::<T>::InvalidPolicy
        );
        ensure!(
            !Authorizers::contains_key(&id),
            TrustError::<T>::AuthorizerExists
        );
        ensure!(
            T::MaxControllers::get() >= new_authorizer.policy.len(),
            TrustError::<T>::TooManyControllers
        );

        // execute
        Authorizers::insert(&id, new_authorizer);

        deposit_indexed_event!(AuthorizerAdded(id));
        Ok(())
    }

    pub(super) fn add_policy_controller_(
        AddPolicyControllerRaw {
            authorizer_id,
            controller,
            ..
        }: AddPolicyControllerRaw<T>,
        authorizer: &mut Authorizer,
    ) -> DispatchResult {
        // check
        ensure!(
            Authorizers::contains_key(&authorizer_id),
            TrustError::<T>::AuthorizerNotExists
        );
        ensure!(
            T::MaxControllers::get() >= authorizer.policy.len() + controller.len() as u32,
            TrustError::<T>::TooManyControllers
        );

        // execute
        match &mut authorizer.policy {
            Policy::OneOf(controllers) => {
                for controller_id in &controller {
                    if !controllers.contains(controller_id) {
                        controllers.insert(*controller_id);
                    }
                }
            }
        }

        deposit_indexed_event!(ControllerAdded(authorizer_id));
        Ok(())
    }

    pub(super) fn remove_policy_controller_(
        RemovePolicyControllerRaw {
            authorizer_id,
            controller,
            ..
        }: RemovePolicyControllerRaw<T>,
        authorizer: &mut Authorizer,
    ) -> DispatchResult {
        // check
        ensure!(
            Authorizers::contains_key(&authorizer_id),
            TrustError::<T>::AuthorizerNotExists
        );

        // execute
        match &mut authorizer.policy {
            Policy::OneOf(controllers) => {
                for controller_id in &controller {
                    if controllers.contains(controller_id) {
                        controllers.remove(controller_id);
                    }
                }
            }
        }

        deposit_indexed_event!(ControllerRemoved(authorizer_id));
        Ok(())
    }

    pub(super) fn add_issuer_(
        AddIssuerRaw {
            authorizer_id,
            entity_ids,
            ..
        }: AddIssuerRaw<T>,
        _: &mut Authorizer,
    ) -> DispatchResult {
        // execute
        for cred_id in &entity_ids {
            Issuers::insert(&authorizer_id, cred_id, ());
        }

        deposit_indexed_event!(IssuerAdded(authorizer_id));
        Ok(())
    }

    pub(super) fn remove_issuer_(
        RemoveIssuerRaw {
            entity_ids,
            authorizer_id,
            ..
        }: RemoveIssuerRaw<T>,
        authorizer: &mut Authorizer,
    ) -> DispatchResult {
        ensure!(!authorizer.add_only, TrustError::<T>::AddOnly);

        // execute
        for cred_id in &entity_ids {
            Issuers::remove(&authorizer_id, cred_id);
        }

        deposit_indexed_event!(IssuerRemoved(authorizer_id));
        Ok(())
    }

    pub(super) fn add_verifier_(
        AddVerifierRaw {
            authorizer_id,
            entity_ids,
            ..
        }: AddVerifierRaw<T>,
        _: &mut Authorizer,
    ) -> DispatchResult {
        // execute
        for cred_id in &entity_ids {
            Verifiers::insert(&authorizer_id, cred_id, ());
        }

        deposit_indexed_event!(VerifierAdded(authorizer_id));
        Ok(())
    }

    pub(super) fn remove_verifier_(
        RemoveVerifierRaw {
            entity_ids,
            authorizer_id,
            ..
        }: RemoveVerifierRaw<T>,
        authorizer: &mut Authorizer,
    ) -> DispatchResult {
        ensure!(!authorizer.add_only, TrustError::<T>::AddOnly);

        // execute
        for cred_id in &entity_ids {
            Verifiers::remove(&authorizer_id, cred_id);
        }

        deposit_indexed_event!(VerifierRemoved(authorizer_id));
        Ok(())
    }

    pub(super) fn remove_authorizer_(
        RemoveAuthorizerRaw { authorizer_id, .. }: RemoveAuthorizerRaw<T>,
        authorizer: &mut Option<Authorizer>,
    ) -> DispatchResult {
        let authorizer = authorizer.take().unwrap();
        ensure!(!authorizer.add_only, TrustError::<T>::AddOnly);

        // execute
        // TODO: limit and cursor
        let _ = Issuers::clear_prefix(&authorizer_id, u32::MAX, None);
        let _ = Verifiers::clear_prefix(&authorizer_id, u32::MAX, None);
        Authorizers::remove(&authorizer_id);

        deposit_indexed_event!(AuthorizerRemoved(authorizer_id));
        Ok(())
    }

    /// Executes action over target authorizer providing a mutable reference if all checks succeed.
    ///
    /// Checks:
    /// 1. Ensure that the authorizer exists and this is not a replayed payload by checking the
    /// equality with stored block number when the authorizer was last modified.
    /// 2. Verify that `proof` authorizes `action` according to `policy`.
    ///
    /// Returns a mutable reference to the underlying authorizer if the command is authorized,
    /// otherwise returns Err.
    pub(crate) fn try_exec_action_over_authorizer<A, F, R, E>(
        action: A,
        proof: Vec<DidSigs<T>>,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, &mut Authorizer) -> Result<R, E>,
        A: Action<T, Target = AuthorizerId>,
        WithNonce<T, A>: ToStateChange<T>,
        E: From<TrustError<T>> + From<did::Error<T>> + From<NonceError>,
    {
        Self::try_exec_removable_action_over_authorizer(action, proof, |action, reg| {
            f(action, reg.as_mut().unwrap())
        })
    }

    /// Executes action over target authorizer providing a mutable reference if all checks succeed.
    ///
    /// Unlike `try_exec_action_over_authorizer`, this action may result in a removal of a
    /// authorizer, if the value under option will be taken.
    ///
    /// Checks:
    /// 1. Ensure that the authorizer exists and this is not a replayed payload by checking the
    /// equality with stored block number when the authorizer was last modified.
    /// 2. Verify that `proof` authorizes `action` according to `policy`.
    ///
    /// Returns a mutable reference to the underlying authorizer wrapped into an option if the
    /// command is authorized, otherwise returns Err.
    pub(crate) fn try_exec_removable_action_over_authorizer<A, F, R, E>(
        mut action: A,
        proof: Vec<DidSigs<T>>,
        f: F,
    ) -> Result<R, E>
    where
        F: FnOnce(A, &mut Option<Authorizer>) -> Result<R, E>,
        A: Action<T, Target = AuthorizerId>,
        WithNonce<T, A>: ToStateChange<T>,
        E: From<TrustError<T>> + From<did::Error<T>> + From<NonceError>,
    {
        ensure!(!action.is_empty(), TrustError::<T>::EmptyPayload);

        Authorizers::try_mutate_exists(action.target(), |authorizer_opt| {
            let authorizer = authorizer_opt.take().ok_or(TrustError::<T>::NoAuthorizer)?;
            // check the signer set satisfies policy
            match &authorizer.policy {
                Policy::OneOf(controllers) => {
                    ensure!(
                        proof.len() == 1 && proof.iter().all(|a| controllers.contains(&a.sig.did)),
                        TrustError::<T>::NotAuthorized
                    );
                }
            }

            let mut new_did_details = Vec::with_capacity(proof.len());
            // check each signature is valid over payload and signed by the claimed signer
            for DidSigs { sig, nonce } in proof {
                let signer = sig.did;

                // Check if nonce is valid and increase it
                let mut did_detail = did::Pallet::<T>::onchain_did_details(&signer)?;
                did_detail
                    .try_update(nonce)
                    .map_err(|_| TrustError::<T>::IncorrectNonce)?;

                let action_with_nonce = WithNonce::new_with_nonce(action, nonce);
                // Verify signature
                let valid = did::Pallet::<T>::verify_sig_from_auth_or_control_key(
                    &action_with_nonce,
                    &sig,
                )?;
                action = action_with_nonce.into_data();

                ensure!(valid, TrustError::<T>::NotAuthorized);
                new_did_details.push((signer, did_detail));
            }

            let mut data_opt = Some(authorizer);
            let res = f(action, &mut data_opt)?;
            *authorizer_opt = data_opt;

            // The nonce of each DID must be updated
            for (signer, did_details) in new_did_details {
                did::Pallet::<T>::insert_did_details(signer, did_details);
            }

            Ok(res)
        })
    }
}
