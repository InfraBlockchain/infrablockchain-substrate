use super::*;

#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(omit_prefix)]
pub struct AddAuthorizer {
    pub id: AuthorizerId,
    pub new_authorizer: Authorizer,
}

#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddPolicyControllerRaw<T> {
    pub authorizer_id: AuthorizerId,
    pub controller: BTreeSet<Did>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemovePolicyControllerRaw<T> {
    pub authorizer_id: AuthorizerId,
    pub controller: BTreeSet<Did>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

/// Command to create a set of issuer withing a authorizer.
/// Creation of issuerless is idempotent; creating a issuer that already exists is
/// allowed, but has no effect.
#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddIssuerRaw<T> {
    /// The authorizer on which to operate
    pub authorizer_id: AuthorizerId,
    /// entity ids which will be added
    pub entity_ids: BTreeSet<TrustedEntityId>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

/// Command to remove a set of issuer within a authorizer.
/// Removal of issuers is idempotent; removing a issuer that doesn't exists is
/// allowed, but has no effect.
#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveIssuerRaw<T> {
    /// The authorizer on which to operate
    pub authorizer_id: AuthorizerId,
    /// entity ids which will be added
    pub entity_ids: BTreeSet<TrustedEntityId>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

/// Command to create a set of verifier withing a authorizer.
/// Creation of verifierless is idempotent; creating a verifier that already exists is
/// allowed, but has no effect.
#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct AddVerifierRaw<T> {
    /// The authorizer on which to operate
    pub authorizer_id: AuthorizerId,
    /// entity ids which will be added
    pub entity_ids: BTreeSet<TrustedEntityId>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

/// Command to remove a set of verifier within a authorizer.
/// Removal of verifiers is idempotent; removing a verifier that doesn't exists is
/// allowed, but has no effect.
#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct RemoveVerifierRaw<T> {
    /// The authorizer on which to operate
    pub authorizer_id: AuthorizerId,
    /// entity ids which will be added
    pub entity_ids: BTreeSet<TrustedEntityId>,
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _marker: PhantomData<T>,
}

/// Command to remove an entire authorizer. Removes all trusted entities in the authorizer as well
/// as authorizer metadata.
#[derive(PartialEq, Eq, Encode, Decode, scale_info_derive::TypeInfo, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    RemoveAuthorizerRaw with 1 as len, authorizer_id as target no_state_change,
    AddPolicyControllerRaw with controller.len() as len, authorizer_id as target no_state_change,
    RemovePolicyControllerRaw with controller.len() as len, authorizer_id as target no_state_change
}

/// Command to create a set of issuer withing a authorizer.
/// Creation of issuerless is idempotent; creating a trusted entities that already exists is
/// allowed, but has no effect.
pub type AddIssuer<T> = WithNonce<T, AddIssuerRaw<T>>;
/// Command to remove a set of issuer within a authorizer.
/// Removal of issuers is idempotent; removing a trusted entities that doesn't exists is
/// allowed, but has no effect.
pub type RemoveIssuer<T> = WithNonce<T, RemoveIssuerRaw<T>>;
/// Command to create a set of verifier withing a authorizer.
/// Creation of verifierless is idempotent; creating a trusted entities that already exists is
/// allowed, but has no effect.
pub type AddVerifier<T> = WithNonce<T, AddVerifierRaw<T>>;
/// Command to remove a set of verifiers within a authorizer.
/// Removal of verifiers is idempotent; removing a trusted entities that doesn't exists is
/// allowed, but has no effect.
pub type RemoveVerifier<T> = WithNonce<T, RemoveVerifierRaw<T>>;
/// Command to remove an entire authorizer. Removes all trusted entitiess in the authorizer as well
/// as authorizer metadata.
pub type RemoveAuthorizer<T> = WithNonce<T, RemoveAuthorizerRaw<T>>;
/// Command to add a set of controller withing a authorizer.
/// Creation of verifierless is idempotent; creating a controller that already exists is
/// allowed, but has no effect.
pub type AddPolicyController<T> = WithNonce<T, AddPolicyControllerRaw<T>>;
/// Command to remove a set of controller withing a authorizer.
/// Creation of verifierless is idempotent; creating a controller that already exists is
/// allowed, but has no effect.
pub type RemovePolicyController<T> = WithNonce<T, RemovePolicyControllerRaw<T>>;

crate::impl_action_with_nonce! {
    for AuthorizerId:
    AddIssuer with data().len() as len, data().authorizer_id as target,
    RemoveIssuer with data().len() as len, data().authorizer_id as target,
    AddVerifier with data().len() as len, data().authorizer_id as target,
    RemoveVerifier with data().len() as len, data().authorizer_id as target,
    RemoveAuthorizer with data().len() as len, data().authorizer_id as target,
    AddPolicyController with data().len() as len, data().authorizer_id as target,
    RemovePolicyController with data().len() as len, data().authorizer_id as target
}
