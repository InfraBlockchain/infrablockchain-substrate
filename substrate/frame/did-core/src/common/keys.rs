use crate::util::Bytes32;
use codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{ed25519, sr25519};

/// An abstraction for a public key. Abstracts the type and value of the public key where the value
/// is a byte array
#[derive(
	Encode,
	Decode,
	Debug,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	MaxEncodedLen,
	scale_info_derive::TypeInfo,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(omit_prefix)]
pub enum PublicKey {
	/// Public key for Sr25519 is 32 bytes
	Sr25519(Bytes32),
	/// Public key for Ed25519 is 32 bytes
	Ed25519(Bytes32),
	/// Compressed X25519 public key, 32 bytes. This key is not used for signing
	X25519(Bytes32),
}

impl From<ed25519::Public> for PublicKey {
	fn from(value: ed25519::Public) -> Self {
		PublicKey::ed25519(value.into())
	}
}

impl From<sr25519::Public> for PublicKey {
	fn from(value: sr25519::Public) -> Self {
		PublicKey::sr25519(value.into())
	}
}

impl PublicKey {
	pub const fn can_sign(&self) -> bool {
		!matches!(self, PublicKey::X25519(_))
	}

	pub const fn sr25519(bytes: [u8; 32]) -> Self {
		PublicKey::Sr25519(Bytes32(bytes))
	}

	pub const fn ed25519(bytes: [u8; 32]) -> Self {
		PublicKey::Ed25519(Bytes32(bytes))
	}

	pub const fn x25519(bytes: [u8; 32]) -> Self {
		PublicKey::X25519(Bytes32(bytes))
	}

	pub fn as_slice(&self) -> &[u8] {
		match self {
			Self::Sr25519(bytes) => &bytes[..],
			Self::Ed25519(bytes) => &bytes[..],
			Self::X25519(bytes) => &bytes[..],
		}
	}
}
