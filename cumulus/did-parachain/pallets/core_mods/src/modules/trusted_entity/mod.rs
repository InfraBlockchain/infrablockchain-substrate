use crate as dock;
use crate::{
    did::{self, Did, DidSignature},
    keys_and_sigs::{SigValue, ED25519_WEIGHT, SR25519_WEIGHT},
    util::{NonceError, WithNonce},
    Action, StorageVersion, ToStateChange,
};
use alloc::collections::BTreeSet;
use codec::{Decode, Encode};
use core::{fmt::Debug, marker::PhantomData};
use sp_std::vec::Vec;

pub use actions::*;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::Get,
    weights::{RuntimeDbWeight, Weight},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use weights::*;

mod actions;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
mod r#impl;
#[cfg(test)]
pub mod tests;
mod weights;

pub trait Config: system::Config + did::Config {
    type RuntimeEvent: From<Event> + Into<<Self as system::Config>::RuntimeEvent>;
    type MaxControllers: Get<u32>;
}

pub type AuthorizerId = [u8; 32];

pub type TrustedEntityId = [u8; 32];

/// Collection of signatures sent by different DIDs.
#[derive(PartialEq, Eq, Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct DidSigs<T>
where
    T: frame_system::Config,
{
    /// Signature by DID
    pub sig: DidSignature<Did>,
    /// Nonce used to make the above signature
    pub nonce: T::BlockNumber,
}

/// Authorization logic for a authorizer.
#[derive(PartialEq, Eq, Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
#[scale_info(omit_prefix)]
pub enum Policy {
    /// Set of dids allowed to modify a authorizer.
    OneOf(BTreeSet<Did>),
}

impl Default for Policy {
    fn default() -> Self {
        Self::OneOf(Default::default())
    }
}

impl Policy {
    /// Check for user error in the construction of self.
    /// if self is invalid, return `false`, else return `true`.
    fn valid(&self) -> bool {
        self.len() != 0
    }

    fn len(&self) -> u32 {
        match self {
            Self::OneOf(controllers) => controllers.len() as u32,
        }
    }
}

/// Metadata about a trusted entity scope.
#[derive(PartialEq, Eq, Encode, Decode, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
#[scale_info(omit_prefix)]
pub struct Authorizer {
    /// Who is allowed to update this authorizer.
    pub policy: Policy,
    pub add_only: bool,
}

/// Return counts of different signature types in given `DidSigs` as 3-Tuple as (no. of Sr22519 sigs,
/// no. of Ed25519 Sigs). Useful for weight calculation and thus the return
/// type is in `Weight` but realistically, it should fit in a u8
fn count_sig_types<T: frame_system::Config>(auth: &[DidSigs<T>]) -> (u64, u64) {
    let mut sr = 0;
    let mut ed = 0;
    for a in auth.iter() {
        match a.sig.sig {
            SigValue::Sr25519(_) => sr += 1,
            SigValue::Ed25519(_) => ed += 1,
        }
    }
    (sr, ed)
}

/// Computes weight of the given `DidSigs`. Considers the no. and types of signatures and no. of reads. Disregards
/// message size as messages are hashed giving the same output size and hashing itself is very cheap.
/// The extrinsic using it might decide to consider adding some weight proportional to the message size.
pub fn get_weight_for_did_sigs<T: frame_system::Config>(
    auth: &[DidSigs<T>],
    db_weights: RuntimeDbWeight,
) -> Weight {
    let (sr, ed) = count_sig_types(auth);
    db_weights
        .reads(auth.len() as u64)
        .saturating_add(SR25519_WEIGHT.saturating_mul(sr))
        .saturating_add(ED25519_WEIGHT.saturating_mul(ed))
}

decl_event!(
    pub enum Event {
        /// Authorizer with given id created
        AuthorizerAdded(AuthorizerId),
        /// Controller with given id Added
        ControllerAdded(AuthorizerId),
        /// Controller with given id Removed
        ControllerRemoved(AuthorizerId),
        /// Issuer were added from given authorizer id
        IssuerAdded(AuthorizerId),
        /// Issuer were removed from given authorizer id
        IssuerRemoved(AuthorizerId),
        /// Verifier were added from given authorizer id
        VerifierAdded(AuthorizerId),
        /// Verifier were removed from given authorizer id
        VerifierRemoved(AuthorizerId),
        /// Authorizer with given id removed
        AuthorizerRemoved(AuthorizerId),
    }
);

decl_error! {
    pub enum TrustError for Module<T: Config> where T: Debug {
        /// The authorization policy provided was illegal.
        InvalidPolicy,
        /// Proof of authorization does not meet policy requirements.
        NotAuthorized,
        /// A authorizer with that name already exists.
        AuthorizerExists,
        /// A authorizer with that name already exists.
        AuthorizerNotExists,
        /// A authorizer with that name does not exist.
        NoAuthorizer,
        /// nonce is incorrect. This is related to replay protection.
        IncorrectNonce,
        /// Too many controllers specified.
        TooManyControllers,
        /// This authorizer is marked as add_only. Deletion of trusted entities is not allowed. Deletion of
        /// the authorizer is not allowed.
        AddOnly,
        /// Action is empty.
        EmptyPayload
    }
}

impl<T: Config + Debug> From<NonceError> for TrustError<T> {
    fn from(NonceError::IncorrectNonce: NonceError) -> Self {
        Self::IncorrectNonce
    }
}

decl_storage! {
    trait Store for Module<T: Config> as TrustedEntity where T: Debug {
        pub(crate) Authorizers get(fn get_authorizer):
            map hasher(blake2_128_concat) dock::trusted_entity::AuthorizerId => Option<Authorizer>;

        Issuers get(fn get_issuer):
            double_map hasher(blake2_128_concat) dock::trusted_entity::AuthorizerId, hasher(opaque_blake2_256) dock::trusted_entity::TrustedEntityId => Option<()>;

        Verifiers get(fn get_verifier):
            double_map hasher(blake2_128_concat) dock::trusted_entity::AuthorizerId, hasher(opaque_blake2_256) dock::trusted_entity::TrustedEntityId => Option<()>;

        pub Version get(fn version): StorageVersion;
    }
    add_extra_genesis {
        build(|_| {
            Version::put(StorageVersion::MultiKey);
        })
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin, T: Debug {
        fn deposit_event() = default;

        type Error = TrustError<T>;

        /// Create a new authrorizer named `id` with `authrorizer` metadata.
        ///
        /// # Errors
        ///
        /// Returns an error if `id` is already in use as a authrorizer id.
        ///
        /// Returns an error if `authrorizer.policy` is invalid.
        #[weight = SubstrateWeight::<T>::new_authorizer(add_authorizer.new_authorizer.policy.len())]
        pub fn new_authorizer(
            origin,
            add_authorizer: AddAuthorizer
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::new_authorizer_(add_authorizer)?;
            Ok(())
        }

        #[weight = 0]
        pub fn add_policy_controller(
            origin,
            controller: dock::trusted_entity::AddPolicyControllerRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(controller, proof, Self::add_policy_controller_)?;
            Ok(())
        }

        #[weight = 0]
        pub fn remove_policy_controller(
            origin,
            controller: dock::trusted_entity::RemovePolicyControllerRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(controller, proof, Self::remove_policy_controller_)?;
            Ok(())
        }
        /// Create some issuer according to the `entity`` command.
        ///
        /// # Errors
        ///
        /// Returns an error if `entity.last_modified` does not match the block number when the
        /// authorizer referenced by `entity.authorizer_id` was last modified.
        ///
        /// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
        /// referenced by `entity.authorizer_id`.
        #[weight = SubstrateWeight::<T>::add_issuer(&proof[0])(entity.len())]
        pub fn add_issuer(
            origin,
            entity: dock::trusted_entity::AddIssuerRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(entity, proof, Self::add_issuer_)?;
            Ok(())
        }

        /// Delete some issuer according to the `entity` command.
        ///
        /// # Errors
        ///
        /// Returns an error if the authorizer referenced by `entity.authorizer_id` is `add_only`.
        ///
        /// Returns an error if `entity.last_modified` does not match the block number when the
        /// authorizer referenced by `authrorizer.authorizer_id` was last modified.
        ///
        /// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
        /// referenced by `entity.authorizer_id`.
        #[weight = SubstrateWeight::<T>::remove_issuer(&proof[0])(entity.len())]
        pub fn remove_issuer(
            origin,
            entity: dock::trusted_entity::RemoveIssuerRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(entity, proof, Self::remove_issuer_)?;
            Ok(())
        }

        /// Create some verifier according to the `entity`` command.
        ///
        /// # Errors
        ///
        /// Returns an error if `entity.last_modified` does not match the block number when the
        /// authorizer referenced by `entity.authorizer_id` was last modified.
        ///
        /// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
        /// referenced by `entity.authorizer_id`.
        #[weight = SubstrateWeight::<T>::add_verifier(&proof[0])(entity.len())]
        pub fn add_verifier(
            origin,
            entity: dock::trusted_entity::AddVerifierRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(entity, proof, Self::add_verifier_)?;
            Ok(())
        }

        /// Delete some verifier according to the `entity` command.
        ///
        /// # Errors
        ///
        /// Returns an error if the authorizer referenced by `entity.authorizer_id` is `add_only`.
        ///
        /// Returns an error if `entity.last_modified` does not match the block number when the
        /// authorizer referenced by `authrorizer.authorizer_id` was last modified.
        ///
        /// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
        /// referenced by `entity.authorizer_id`.
        #[weight = SubstrateWeight::<T>::remove_verifier(&proof[0])(entity.len())]
        pub fn remove_verifier(
            origin,
            entity: dock::trusted_entity::RemoveVerifierRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_action_over_authorizer(entity, proof, Self::remove_verifier_)?;
            Ok(())
        }

        /// Delete an entire authorizer. Deletes all trusted entities within the authorizer, as well as
        /// authorizer metadata. Once the authorizer is deleted, it can be reclaimed by any party using
        /// a call to `new_authorizer`.
        ///
        /// # Errors
        ///
        /// Returns an error if the authorizer referenced by `entity.authorizer_id` is `add_only`.
        ///
        /// Returns an error if `removal.last_modified` does not match the block number when the
        /// authorizer referenced by `removal.authorizer_id` was last modified.
        ///
        /// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
        /// referenced by `removal.authorizer_id`.
        #[weight = SubstrateWeight::<T>::remove_authorizer(&proof[0])]
        pub fn remove_authorizer(
            origin,
            removal: dock::trusted_entity::RemoveAuthorizerRaw<T>,
            proof: Vec<DidSigs<T>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_exec_removable_action_over_authorizer(removal, proof, Self::remove_authorizer_)?;
            Ok(())
        }
    }
}

impl<T: frame_system::Config> SubstrateWeight<T> {
    fn add_issuer(DidSigs { sig, .. }: &DidSigs<T>) -> fn(u32) -> Weight {
        match sig.sig {
            SigValue::Sr25519(_) => Self::add_issuer_sr25519,
            SigValue::Ed25519(_) => Self::add_issuer_ed25519,
        }
    }

    fn remove_issuer(DidSigs { sig, .. }: &DidSigs<T>) -> fn(u32) -> Weight {
        match sig.sig {
            SigValue::Sr25519(_) => Self::remove_issuer_sr25519,
            SigValue::Ed25519(_) => Self::remove_issuer_ed25519,
        }
    }

    fn add_verifier(DidSigs { sig, .. }: &DidSigs<T>) -> fn(u32) -> Weight {
        match sig.sig {
            SigValue::Sr25519(_) => Self::add_verifier_sr25519,
            SigValue::Ed25519(_) => Self::add_verifier_ed25519,
        }
    }

    fn remove_verifier(DidSigs { sig, .. }: &DidSigs<T>) -> fn(u32) -> Weight {
        match sig.sig {
            SigValue::Sr25519(_) => Self::remove_verifier_sr25519,
            SigValue::Ed25519(_) => Self::remove_verifier_ed25519,
        }
    }

    fn remove_authorizer(DidSigs { sig, .. }: &DidSigs<T>) -> Weight {
        (match sig.sig {
            SigValue::Sr25519(_) => Self::remove_authorizer_sr25519,
            SigValue::Ed25519(_) => Self::remove_authorizer_ed25519,
        }())
    }
}
