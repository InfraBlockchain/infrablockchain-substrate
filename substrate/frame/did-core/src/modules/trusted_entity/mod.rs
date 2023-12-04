#[cfg(feature = "serde")]
use crate::util::hex;
use crate::{
	common::{self, DidSignatureWithNonce, HasPolicy, Limits, Policy, SigValue, ToStateChange},
	did::{self},
	util::{Action, NonceError, WithNonce},
};
use alloc::collections::BTreeSet;
use codec::{Decode, Encode, MaxEncodedLen};
use core::ops::{Index, RangeFull};
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

use frame_support::{dispatch::DispatchResult, ensure, weights::Weight, DebugNoBound};
use frame_system::ensure_signed;
use sp_std::prelude::*;
use weights::*;

pub use actions::*;
pub use pallet::*;

mod actions;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
mod r#impl;
#[cfg(test)]
pub mod tests;
mod weights;

/// Points to an on-chain revocation registry.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Copy, Ord, PartialOrd, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
#[scale_info(omit_prefix)]
pub struct AuthorizerId(#[cfg_attr(feature = "serde", serde(with = "hex"))] pub [u8; 32]);

impl Index<RangeFull> for AuthorizerId {
	type Output = [u8; 32];

	fn index(&self, _: RangeFull) -> &Self::Output {
		&self.0
	}
}

crate::impl_wrapper!(AuthorizerId([u8; 32]));

/// Points to a revocation which may or may not exist in a registry.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Copy, Ord, PartialOrd, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
#[scale_info(omit_prefix)]
pub struct TrustedEntityId(#[cfg_attr(feature = "serde", serde(with = "hex"))] pub [u8; 32]);

impl Index<RangeFull> for TrustedEntityId {
	type Output = [u8; 32];

	fn index(&self, _: RangeFull) -> &Self::Output {
		&self.0
	}
}

crate::impl_wrapper!(TrustedEntityId([u8; 32]));

/// Metadata about a revocation scope.
#[derive(
	PartialEq, Eq, Encode, Decode, Clone, DebugNoBound, MaxEncodedLen, scale_info_derive::TypeInfo,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct Authorizer<T: Limits> {
	/// Who is allowed to update this registry.
	pub policy: Policy<T>,
	/// true: credentials can be add_issuerd, but not un-add_issuerd and the registry can't be
	/// removed either false: credentials can be add_issuerd and un-add_issuerd
	pub add_only: bool,
}

impl<T: Limits> HasPolicy<T> for Authorizer<T> {
	fn policy(&self) -> &Policy<T> {
		&self.policy
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + did::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>
			+ Into<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	pub enum Event {
		/// Authorizer with given id created
		AuthorizerAdded(AuthorizerId),
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

	/// Revocation Error
	#[pallet::error]
	pub enum Error<T> {
		/// A revocation registry with that name already exists.
		RegExists,
		/// nonce is incorrect. This is related to replay protection.
		IncorrectNonce,
		/// Too many controllers specified.
		TooManyControllers,
		/// This registry is marked as add_only. Deletion of revocations is not allowed. Deletion
		/// of the registry is not allowed.
		AddOnly,
		/// Action is empty.
		EmptyPayload,
	}

	impl<T: Config> From<NonceError> for Error<T> {
		fn from(NonceError::IncorrectNonce: NonceError) -> Self {
			Self::IncorrectNonce
		}
	}

	/// Registry metadata
	#[pallet::storage]
	#[pallet::getter(fn get_authorizer)]
	pub type Authorizers<T: Config> = StorageMap<_, Blake2_128Concat, AuthorizerId, Authorizer<T>>;

	/// The single global revocation set
	// double_map requires and explicit hasher specification for the second key. blake2_256 is
	// the default.
	#[pallet::storage]
	#[pallet::getter(fn get_issuer)]
	pub type Issuers<T> =
		StorageDoubleMap<_, Blake2_128Concat, AuthorizerId, Blake2_256, TrustedEntityId, ()>;

	/// The single global revocation set
	// double_map requires and explicit hasher specification for the second key. blake2_256 is
	// the default.
	#[pallet::storage]
	#[pallet::getter(fn get_verifier)]
	pub type Verifiers<T> =
		StorageDoubleMap<_, Blake2_128Concat, AuthorizerId, Blake2_256, TrustedEntityId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn version)]
	pub type Version<T> = StorageValue<_, common::StorageVersion, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub _marker: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _marker: PhantomData }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Version::<T>::put(common::StorageVersion::MultiKey);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new revocation registry named `id` with `registry` metadata.
		///
		/// # Errors
		///
		/// Returns an error if `id` is already in use as a registry id.
		///
		/// Returns an error if `registry.policy` is invalid.
		#[pallet::weight(SubstrateWeight::<T>::new_authorizer(add_authorizer.new_authorizer.policy.len()))]
		#[pallet::call_index(0)]
		pub fn new_authorizer(
			origin: OriginFor<T>,
			add_authorizer: AddAuthorizer<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::new_authorizer_(add_authorizer)?;
			Ok(())
		}

		/// Create some revocations according to the `add_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if `add_issuer.last_modified` does not match the block number when the
		/// registry referenced by `add_issuer.registry_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the registry
		/// referenced by `add_issuer.registry_id`.
		#[pallet::weight(SubstrateWeight::<T>::add_issuer(&proof[0])(add_issuer.len()))]
		#[pallet::call_index(1)]
		pub fn add_issuer(
			origin: OriginFor<T>,
			add_issuer: AddIssuerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_registry(Self::add_issuer_, add_issuer, proof)?;
			Ok(())
		}

		/// Delete some revocations according to the `remove_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if the registry referenced by `add_issuer.registry_id` is `add_only`.
		///
		/// Returns an error if `remove_issuer.last_modified` does not match the block number when
		/// the registry referenced by `add_issuer.registry_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the registry
		/// referenced by `remove_issuer.registry_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_issuer(&proof[0])(remove_issuer.len()))]
		#[pallet::call_index(2)]
		pub fn remove_issuer(
			origin: OriginFor<T>,
			remove_issuer: RemoveIssuerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_registry(Self::remove_issuer_, remove_issuer, proof)?;
			Ok(())
		}

		/// Delete some revocations according to the `remove_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if the registry referenced by `add_issuer.registry_id` is `add_only`.
		///
		/// Returns an error if `remove_issuer.last_modified` does not match the block number when
		/// the registry referenced by `add_issuer.registry_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the registry
		/// referenced by `remove_issuer.registry_id`.
		#[pallet::weight(SubstrateWeight::<T>::add_verifier(&proof[0])(add_verifier.len()))]
		#[pallet::call_index(3)]
		pub fn add_verifier(
			origin: OriginFor<T>,
			add_verifier: AddVerifierRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_registry(Self::add_verifier_, add_verifier, proof)?;
			Ok(())
		}

		/// Create some revocations according to the `add_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if `add_issuer.last_modified` does not match the block number when the
		/// registry referenced by `add_issuer.registry_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the registry
		/// referenced by `add_issuer.registry_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_verifier(&proof[0])(remove_verifier.len()))]
		#[pallet::call_index(4)]
		pub fn remove_verifier(
			origin: OriginFor<T>,
			remove_verifier: RemoveVerifierRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_registry(Self::remove_verifier_, remove_verifier, proof)?;
			Ok(())
		}

		/// Delete an entire registry. Deletes all revocations within the registry, as well as
		/// registry metadata. Once the registry is deleted, it can be reclaimed by any party using
		/// a call to `new_registry`.
		///
		/// # Errors
		///
		/// Returns an error if the registry referenced by `add_issuer.registry_id` is `add_only`.
		///
		/// Returns an error if `removal.last_modified` does not match the block number when the
		/// registry referenced by `removal.registry_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the registry
		/// referenced by `removal.registry_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_authorizer(&proof[0]))]
		#[pallet::call_index(5)]
		pub fn remove_authorizer(
			origin: OriginFor<T>,
			removal: RemoveAuthorizerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_removable_action_over_registry(
				Self::remove_authorizer_,
				removal,
				proof,
			)?;
			Ok(())
		}
	}
}

impl<T: Config> SubstrateWeight<T> {
	fn add_issuer(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::add_issuer_sr25519,
			SigValue::Ed25519(_) => Self::add_issuer_ed25519,
			SigValue::Secp256k1(_) => Self::add_issuer_secp256k1,
		}
	}

	fn remove_issuer(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::remove_issuer_sr25519,
			SigValue::Ed25519(_) => Self::remove_issuer_ed25519,
			SigValue::Secp256k1(_) => Self::remove_issuer_secp256k1,
		}
	}

	fn add_verifier(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::add_verifier_sr25519,
			SigValue::Ed25519(_) => Self::add_verifier_ed25519,
			SigValue::Secp256k1(_) => Self::add_verifier_secp256k1,
		}
	}

	fn remove_verifier(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::remove_verifier_sr25519,
			SigValue::Ed25519(_) => Self::remove_verifier_ed25519,
			SigValue::Secp256k1(_) => Self::remove_verifier_secp256k1,
		}
	}

	fn remove_authorizer(DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>) -> Weight {
		(match sig.sig {
			SigValue::Sr25519(_) => Self::remove_authorizer_sr25519,
			SigValue::Ed25519(_) => Self::remove_authorizer_ed25519,
			SigValue::Secp256k1(_) => Self::remove_authorizer_secp256k1,
		}())
	}
}
