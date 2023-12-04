use super::*;
use frame_support::DebugNoBound;

#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddAuthorizer<T: Limits> {
	pub id: AuthorizerId,
	pub new_authorizer: Authorizer<T>,
}

/// Command to create a set of revocations withing a registry.
/// Creation of revocations is idempotent; creating a revocation that already exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddIssuerRaw<T> {
	/// The registry on which to operate
	pub authorizer_id: AuthorizerId,
	/// Credential ids which will be revoked
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove a set of revocations within a registry.
/// Removal of revocations is idempotent; removing a revocation that doesn't exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveIssuerRaw<T> {
	/// The registry on which to operate
	pub authorizer_id: AuthorizerId,
	/// Credential ids which will be revoked
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to create a set of revocations withing a registry.
/// Creation of revocations is idempotent; creating a revocation that already exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddVerifierRaw<T> {
	/// The registry on which to operate
	pub authorizer_id: AuthorizerId,
	/// Credential ids which will be revoked
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove a set of revocations within a registry.
/// Removal of revocations is idempotent; removing a revocation that doesn't exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveVerifierRaw<T> {
	/// The registry on which to operate
	pub authorizer_id: AuthorizerId,
	/// Credential ids which will be revoked
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove an entire registry. Removes all revocations in the registry as well as
/// registry metadata.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveAuthorizerRaw<T> {
	/// The registry on which to operate
	pub authorizer_id: AuthorizerId,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

crate::impl_action! {
	for AuthorizerId:
	AddIssuerRaw with entity_ids.len() as len, authorizer_id as target no_state_change,
	RemoveIssuerRaw with entity_ids.len() as len, authorizer_id as target no_state_change,
	AddVerifierRaw with entity_ids.len() as len, authorizer_id as target no_state_change,
	RemoveVerifierRaw with entity_ids.len() as len, authorizer_id as target no_state_change,
	RemoveAuthorizerRaw with 1 as len, authorizer_id as target no_state_change
}

/// Command to create a set of revocations withing a registry.
/// Creation of revocations is idempotent; creating a revocation that already exists is allowed,
/// but has no effect.
pub type AddIssuer<T> = WithNonce<T, AddIssuerRaw<T>>;
/// Command to remove a set of revocations within a registry.
/// Removal of revocations is idempotent; removing a revocation that doesn't exists is allowed,
/// but has no effect.
pub type RemoveIssuer<T> = WithNonce<T, RemoveIssuerRaw<T>>;
/// Command to create a set of revocations withing a registry.
/// Creation of revocations is idempotent; creating a revocation that already exists is allowed,
/// but has no effect.
pub type AddVerifier<T> = WithNonce<T, AddVerifierRaw<T>>;
/// Command to remove a set of revocations within a registry.
/// Removal of revocations is idempotent; removing a revocation that doesn't exists is allowed,
/// but has no effect.
pub type RemoveVerifier<T> = WithNonce<T, RemoveVerifierRaw<T>>;
/// Command to remove an entire registry. Removes all revocations in the registry as well as
/// registry metadata.
pub type RemoveAuthorizer<T> = WithNonce<T, RemoveAuthorizerRaw<T>>;

crate::impl_action_with_nonce! {
	for AuthorizerId:
	AddIssuer with data().len() as len, data().authorizer_id as target,
	RemoveIssuer with data().len() as len, data().authorizer_id as target,
	AddVerifier with data().len() as len, data().authorizer_id as target,
	RemoveVerifier with data().len() as len, data().authorizer_id as target,
	RemoveAuthorizer with data().len() as len, data().authorizer_id as target
}
