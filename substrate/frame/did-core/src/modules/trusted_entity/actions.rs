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

/// Command to create a set of issuers withing a authorizer.
/// Creation of issuers is idempotent; creating a issuers that already exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddIssuerRaw<T> {
	/// The authorizer on which to operate
	pub authorizer_id: AuthorizerId,
	/// entity ids which will be registered as trusted entities
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove a set of issuers within a authorizer.
/// Removal of issuers is idempotent; removing a issuers that doesn't exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveIssuerRaw<T> {
	/// The authorizer on which to operate
	pub authorizer_id: AuthorizerId,
	/// entity ids which will be removed as trusted entities
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to create a set of verifiers withing a authorizer.
/// Creation of verifiers is idempotent; creating a verifiers that already exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddVerifierRaw<T> {
	/// The authorizer on which to operate
	pub authorizer_id: AuthorizerId,
	/// entity ids which will be registered as trusted entities
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove a set of verifiers within a authorizer.
/// Removal of verifiers is idempotent; removing a verifiers that doesn't exists is allowed,
/// but has no effect.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveVerifierRaw<T> {
	/// The authorizer on which to operate
	pub authorizer_id: AuthorizerId,
	/// entity ids which will be removed as trusted entities
	pub entity_ids: BTreeSet<TrustedEntityId>,
	#[codec(skip)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub _marker: PhantomData<T>,
}

/// Command to remove an entire authorizer storage. Removes all issuers and verifiers in the
/// authorizer storage as well as authorizer metadata.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Clone, PartialEq, DebugNoBound, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveAuthorizerRaw<T> {
	/// The authorizer on which to operate
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

/// Command to create a set of issuers withing a authorizer.
/// Creation of issuers is idempotent; creating a issuers that already exists is allowed,
/// but has no effect.
pub type AddIssuer<T> = WithNonce<T, AddIssuerRaw<T>>;
/// Command to remove a set of issuers within a authorizer.
/// Removal of issuers is idempotent; removing a issuers that doesn't exists is allowed,
/// but has no effect.
pub type RemoveIssuer<T> = WithNonce<T, RemoveIssuerRaw<T>>;
/// Command to create a set of verifiers withing a authorizer.
/// Creation of verifiers is idempotent; creating a verifiers that already exists is allowed,
/// but has no effect.
pub type AddVerifier<T> = WithNonce<T, AddVerifierRaw<T>>;
/// Command to remove a set of verifiers within a authorizer.
/// Removal of verifiers is idempotent; removing a verifiers that doesn't exists is allowed,
/// but has no effect.
pub type RemoveVerifier<T> = WithNonce<T, RemoveVerifierRaw<T>>;
/// Command to remove an entire authorizer storage. Removes all issuers and verifiers in the
/// authorizer storage as well as authorizer metadata.
pub type RemoveAuthorizer<T> = WithNonce<T, RemoveAuthorizerRaw<T>>;

crate::impl_action_with_nonce! {
	for AuthorizerId:
	AddIssuer with data().len() as len, data().authorizer_id as target,
	RemoveIssuer with data().len() as len, data().authorizer_id as target,
	AddVerifier with data().len() as len, data().authorizer_id as target,
	RemoveVerifier with data().len() as len, data().authorizer_id as target,
	RemoveAuthorizer with data().len() as len, data().authorizer_id as target
}
