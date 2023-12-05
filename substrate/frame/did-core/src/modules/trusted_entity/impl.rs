use super::*;
use crate::{
	common::{DidSignatureWithNonce, PolicyExecutionError},
	deposit_indexed_event,
};

impl<T: Config> Pallet<T> {
	pub(super) fn new_authorizer_(
		AddAuthorizer { new_authorizer, id }: AddAuthorizer<T>,
	) -> DispatchResult {
		// check
		new_authorizer.policy.ensure_valid()?;
		ensure!(!Authorizers::<T>::contains_key(id), Error::<T>::AuthzExists);

		// execute
		Authorizers::<T>::insert(id, new_authorizer);

		deposit_indexed_event!(AuthorizerAdded(id));
		Ok(())
	}

	pub(super) fn add_issuer_(
		AddIssuerRaw { authorizer_id, entity_ids, .. }: AddIssuerRaw<T>,
		_: &mut Authorizer<T>,
	) -> DispatchResult {
		// execute
		for cred_id in &entity_ids {
			Issuers::<T>::insert(authorizer_id, cred_id, ());
		}

		deposit_indexed_event!(IssuerAdded(authorizer_id));
		Ok(())
	}

	pub(super) fn remove_issuer_(
		RemoveIssuerRaw { entity_ids, authorizer_id, .. }: RemoveIssuerRaw<T>,
		authorizer: &mut Authorizer<T>,
	) -> DispatchResult {
		ensure!(!authorizer.add_only, Error::<T>::AddOnly);

		// execute
		for cred_id in &entity_ids {
			Issuers::<T>::remove(authorizer_id, cred_id);
		}

		deposit_indexed_event!(IssuerRemoved(authorizer_id));
		Ok(())
	}

	pub(super) fn add_verifier_(
		AddVerifierRaw { authorizer_id, entity_ids, .. }: AddVerifierRaw<T>,
		_: &mut Authorizer<T>,
	) -> DispatchResult {
		// execute
		for cred_id in &entity_ids {
			Verifiers::<T>::insert(authorizer_id, cred_id, ());
		}

		deposit_indexed_event!(VerifierAdded(authorizer_id));
		Ok(())
	}

	pub(super) fn remove_verifier_(
		RemoveVerifierRaw { entity_ids, authorizer_id, .. }: RemoveVerifierRaw<T>,
		authorizer: &mut Authorizer<T>,
	) -> DispatchResult {
		ensure!(!authorizer.add_only, Error::<T>::AddOnly);

		// execute
		for cred_id in &entity_ids {
			Verifiers::<T>::remove(authorizer_id, cred_id);
		}

		deposit_indexed_event!(VerifierRemoved(authorizer_id));
		Ok(())
	}

	pub(super) fn remove_authorizer_(
		RemoveAuthorizerRaw { authorizer_id, .. }: RemoveAuthorizerRaw<T>,
		authorizer: &mut Option<Authorizer<T>>,
	) -> DispatchResult {
		let authorizer = authorizer.take().unwrap();
		ensure!(!authorizer.add_only, Error::<T>::AddOnly);

		// execute
		// TODO: limit and cursor
		let _ = Issuers::<T>::clear_prefix(authorizer_id, u32::MAX, None);
		let _ = Verifiers::<T>::clear_prefix(authorizer_id, u32::MAX, None);
		Authorizers::<T>::remove(authorizer_id);

		deposit_indexed_event!(AuthorizerRemoved(authorizer_id));
		Ok(())
	}

	/// Executes action over target authorizer providing a mutable reference if all checks succeed.
	///
	/// Checks:
	/// 1. Ensure that the `Authorizer` exists.
	/// 2. Verify that `proof` authorizes `action` according to `policy`.
	/// 3. Verify that the action is not a replayed payload by ensuring each provided controller
	/// nonce equals the last nonce plus 1.
	///
	/// Returns a mutable reference to the underlying authorizer if the command is authorized,
	/// otherwise returns Err.
	pub(crate) fn try_exec_action_over_authorizer<A, F, R, E>(
		f: F,
		action: A,
		proof: Vec<DidSignatureWithNonce<T>>,
	) -> Result<R, E>
	where
		F: FnOnce(A, &mut Authorizer<T>) -> Result<R, E>,
		A: Action<Target = AuthorizerId>,
		WithNonce<T, A>: ToStateChange<T>,
		E: From<Error<T>> + From<PolicyExecutionError> + From<did::Error<T>> + From<NonceError>,
	{
		Self::try_exec_removable_action_over_authorizer(
			|action, reg| f(action, reg.as_mut().unwrap()),
			action,
			proof,
		)
	}

	/// Executes action over target authorizer providing a mutable reference if all checks succeed.
	///
	/// Unlike `try_exec_action_over_authorizer`, this action may result in a removal of a
	/// Authorizer, if the value under option will be taken.
	///
	/// Checks:
	/// 1. Ensure that the `Authorizer` exists.
	/// 2. Verify that `proof` authorizes `action` according to `policy`.
	/// 3. Verify that the action is not a replayed payload by ensuring each provided controller
	/// nonce equals the last nonce plus 1.
	///
	/// Returns a mutable reference to the underlying authorizer wrapped into an option if the
	/// command is authorized, otherwise returns Err.
	pub(crate) fn try_exec_removable_action_over_authorizer<A, F, R, E>(
		f: F,
		action: A,
		proof: Vec<DidSignatureWithNonce<T>>,
	) -> Result<R, E>
	where
		F: FnOnce(A, &mut Option<Authorizer<T>>) -> Result<R, E>,
		A: Action<Target = AuthorizerId>,
		WithNonce<T, A>: ToStateChange<T>,
		E: From<Error<T>> + From<PolicyExecutionError> + From<did::Error<T>> + From<NonceError>,
	{
		ensure!(!action.is_empty(), Error::EmptyPayload);

		Authorizers::try_mutate_exists(action.target(), |authorizer| {
			Policy::try_exec_removable_action(authorizer, f, action, proof)
		})
	}
}
