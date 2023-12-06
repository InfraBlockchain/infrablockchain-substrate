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

/// Points to an on-chain authorizer.
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

/// Points to a issuer or verifier which may or may not exist in a authorizer.
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

/// Metadata about a authorizer scope.
#[derive(
	PartialEq, Eq, Encode, Decode, Clone, DebugNoBound, MaxEncodedLen, scale_info_derive::TypeInfo,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "T: Sized", deserialize = "T: Sized")))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[scale_info(skip_type_params(T))]
#[scale_info(omit_prefix)]
pub struct Authorizer<T: Limits> {
	/// Who is allowed to update this authorizer.
	pub policy: Policy<T>,
	/// true: credentials can be add entities, but not remove entities and the authorizer can't be
	/// removed either false: credentials can be add entities and remove entities
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
		/// A authorizer with that name already exists.
		AuthzExists,
		/// nonce is incorrect. This is related to replay protection.
		IncorrectNonce,
		/// Too many controllers specified.
		TooManyControllers,
		/// This authorizer is marked as add_only. Deletion of revocations is not allowed. Deletion
		/// of the authorizer is not allowed.
		AddOnly,
		/// Action is empty.
		EmptyPayload,
	}

	impl<T: Config> From<NonceError> for Error<T> {
		fn from(NonceError::IncorrectNonce: NonceError) -> Self {
			Self::IncorrectNonce
		}
	}

	/// Authorizer metadata
	#[pallet::storage]
	#[pallet::getter(fn get_authorizer)]
	pub type Authorizers<T: Config> = StorageMap<_, Blake2_128Concat, AuthorizerId, Authorizer<T>>;

	/// The single global issuer set
	// double_map requires and explicit hasher specification for the second key. blake2_256 is
	// the default.
	#[pallet::storage]
	#[pallet::getter(fn get_issuer)]
	pub type Issuers<T> =
		StorageDoubleMap<_, Blake2_128Concat, AuthorizerId, Blake2_256, TrustedEntityId, ()>;

	/// The single global verifier set
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
		/// Create a new authorizer named `id` with `authorizer` metadata.
		///
		/// # Errors
		///
		/// Returns an error if `id` is already in use as a authorizer id.
		///
		/// Returns an error if `authorizer.policy` is invalid.
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

		/// Create some issuer according to the `add_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if `add_issuer.last_modified` does not match the block number when the
		/// authorizer referenced by `add_issuer.authorizer_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
		/// referenced by `add_issuer.authorizer_id`.
		#[pallet::weight(SubstrateWeight::<T>::add_issuer(&proof[0])(add_issuer.len()))]
		#[pallet::call_index(1)]
		pub fn add_issuer(
			origin: OriginFor<T>,
			add_issuer: AddIssuerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_authorizer(Self::add_issuer_, add_issuer, proof)?;
			Ok(())
		}

		/// Delete some issuer according to the `remove_issuer` command.
		///
		/// # Errors
		///
		/// Returns an error if the authorizer referenced by `add_issuer.authorizer_id` is
		/// `add_only`.
		///
		/// Returns an error if `remove_issuer.last_modified` does not match the block number when
		/// the authorizer referenced by `add_issuer.authorizer_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
		/// referenced by `remove_issuer.authorizer_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_issuer(&proof[0])(remove_issuer.len()))]
		#[pallet::call_index(2)]
		pub fn remove_issuer(
			origin: OriginFor<T>,
			remove_issuer: RemoveIssuerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_authorizer(Self::remove_issuer_, remove_issuer, proof)?;
			Ok(())
		}

		/// Create some verifier according to the `add_verifier` command.
		///
		/// # Errors
		///
		/// Returns an error if `add_verifier.last_modified` does not match the block number when
		/// the authorizer referenced by `add_verifier.authorizer_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
		/// referenced by `add_verifier.authorizer_id`.
		#[pallet::weight(SubstrateWeight::<T>::add_verifier(&proof[0])(add_verifier.len()))]
		#[pallet::call_index(3)]
		pub fn add_verifier(
			origin: OriginFor<T>,
			add_verifier: AddVerifierRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_authorizer(Self::add_verifier_, add_verifier, proof)?;
			Ok(())
		}

		/// Delete some verifier according to the `remove_verifier` command.
		///
		/// # Errors
		///
		/// Returns an error if the authorizer referenced by `add_issuer.authorizer_id` is
		/// `add_only`.
		///
		/// Returns an error if `remove_verifier.last_modified` does not match the block number when
		/// the authorizer referenced by `add_verifier.authorizer_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
		/// referenced by `remove_verifier.authorizer_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_verifier(&proof[0])(remove_verifier.len()))]
		#[pallet::call_index(4)]
		pub fn remove_verifier(
			origin: OriginFor<T>,
			remove_verifier: RemoveVerifierRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_action_over_authorizer(Self::remove_verifier_, remove_verifier, proof)?;
			Ok(())
		}

		/// Delete an entire authorizer. Deletes all issuer, verifier within the authorizer, as well
		/// as authorizer metadata. Once the authorizer is deleted, it can be reclaimed by any party
		/// using a call to `new_authorizer`.
		///
		/// # Errors
		///
		/// Returns an error if the authorizer referenced by `add_issuer.authorizer_id` is
		/// `add_only`.
		///
		/// Returns an error if `removal.last_modified` does not match the block number when the
		/// authorizer referenced by `removal.authorizer_id` was last modified.
		///
		/// Returns an error if `proof` does not satisfy the policy requirements of the authorizer
		/// referenced by `removal.authorizer_id`.
		#[pallet::weight(SubstrateWeight::<T>::remove_authorizer(&proof[0]))]
		#[pallet::call_index(5)]
		pub fn remove_authorizer(
			origin: OriginFor<T>,
			removal: RemoveAuthorizerRaw<T>,
			proof: Vec<DidSignatureWithNonce<T>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::try_exec_removable_action_over_authorizer(
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
		}
	}

	fn remove_issuer(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::remove_issuer_sr25519,
			SigValue::Ed25519(_) => Self::remove_issuer_ed25519,
		}
	}

	fn add_verifier(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::add_verifier_sr25519,
			SigValue::Ed25519(_) => Self::add_verifier_ed25519,
		}
	}

	fn remove_verifier(
		DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>,
	) -> fn(u32) -> Weight {
		match sig.sig {
			SigValue::Sr25519(_) => Self::remove_verifier_sr25519,
			SigValue::Ed25519(_) => Self::remove_verifier_ed25519,
		}
	}

	fn remove_authorizer(DidSignatureWithNonce { sig, .. }: &DidSignatureWithNonce<T>) -> Weight {
		(match sig.sig {
			SigValue::Sr25519(_) => Self::remove_authorizer_sr25519,
			SigValue::Ed25519(_) => Self::remove_authorizer_ed25519,
		}())
	}
}
