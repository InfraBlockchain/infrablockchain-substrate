use crate::util::{Bytes32, Bytes64};
use codec::{Decode, Encode};
use frame_support::dispatch::Weight;
use sp_core::{ed25519, sr25519, Pair};
use sp_runtime::traits::Verify;

/// An abstraction for a public key. Abstracts the type and value of the public key where the value is a
/// byte array
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(scale_info_derive::TypeInfo)]
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
    fn from(ed25519::Public(pubkey): ed25519::Public) -> Self {
        PublicKey::ed25519(pubkey)
    }
}

impl From<sr25519::Public> for PublicKey {
    fn from(sr25519::Public(pubkey): sr25519::Public) -> Self {
        PublicKey::sr25519(pubkey)
    }
}

/// An abstraction for a signature.
#[derive(Encode, Decode, scale_info_derive::TypeInfo, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[scale_info(omit_prefix)]
pub enum SigValue {
    /// Signature for Sr25519 is 64 bytes
    Sr25519(Bytes64),
    /// Signature for Ed25519 is 64 bytes
    Ed25519(Bytes64),
}

impl PublicKey {
    pub const fn can_sign(&self) -> bool {
        match self {
            PublicKey::X25519(_) => false,
            _ => true,
        }
    }

    pub const fn sr25519(bytes: [u8; 32]) -> Self {
        PublicKey::Sr25519(Bytes32 { value: bytes })
    }

    pub const fn ed25519(bytes: [u8; 32]) -> Self {
        PublicKey::Ed25519(Bytes32 { value: bytes })
    }

    pub const fn x25519(bytes: [u8; 32]) -> Self {
        PublicKey::X25519(Bytes32 { value: bytes })
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Sr25519(bytes) => &bytes.value[..],
            Self::Ed25519(bytes) => &bytes.value[..],
            Self::X25519(bytes) => &bytes.value[..],
        }
    }
}

impl From<ed25519::Signature> for SigValue {
    fn from(ed25519::Signature(sig): ed25519::Signature) -> Self {
        SigValue::Ed25519(sig.into())
    }
}

impl From<sr25519::Signature> for SigValue {
    fn from(sr25519::Signature(sig): sr25519::Signature) -> Self {
        SigValue::Sr25519(sig.into())
    }
}

impl SigValue {
    /// Try to get reference to the bytes if its a Sr25519 signature. Return error if its not.
    pub fn as_sr25519_sig_bytes(&self) -> Result<&[u8], ()> {
        match self {
            SigValue::Sr25519(bytes) => Ok(bytes.as_bytes()),
            _ => Err(()),
        }
    }

    /// Try to get reference to the bytes if its a Ed25519 signature. Return error if its not.
    pub fn as_ed25519_sig_bytes(&self) -> Result<&[u8], ()> {
        match self {
            SigValue::Ed25519(bytes) => Ok(bytes.as_bytes()),
            _ => Err(()),
        }
    }

    /// Get weight for signature verification.
    /// Considers the type of signature. Disregards message size as messages are hashed giving the
    /// same output size and hashing itself is very cheap. The extrinsic using it might decide to
    /// consider adding some weight proportional to the message size.
    pub fn weight(&self) -> Weight {
        match self {
            SigValue::Sr25519(_) => SR25519_WEIGHT,
            SigValue::Ed25519(_) => ED25519_WEIGHT,
        }
    }

    pub fn verify(&self, message: &[u8], public_key: &PublicKey) -> Result<bool, ()> {
        macro_rules! verify {
            ( $message:ident, $sig_bytes:ident, $pk_bytes:ident, $sig_type:expr, $pk_type:expr ) => {{
                let signature = $sig_type($sig_bytes.value);
                let pk = $pk_type($pk_bytes.value);
                signature.verify($message, &pk)
            }};
        }
        let result = match (public_key, self) {
            (PublicKey::Sr25519(pk_bytes), SigValue::Sr25519(sig_bytes)) => {
                verify!(
                    message,
                    sig_bytes,
                    pk_bytes,
                    sr25519::Signature,
                    sr25519::Public
                )
            }
            (PublicKey::Ed25519(pk_bytes), SigValue::Ed25519(sig_bytes)) => {
                verify!(
                    message,
                    sig_bytes,
                    pk_bytes,
                    ed25519::Signature,
                    ed25519::Public
                )
            }
            _ => {
                return Err(());
            }
        };
        Ok(result)
    }

    pub fn sr25519(msg: &[u8], pair: &sr25519::Pair) -> Self {
        SigValue::Sr25519(Bytes64 {
            value: pair.sign(msg).0,
        })
    }

    pub fn ed25519(msg: &[u8], pair: &ed25519::Pair) -> Self {
        SigValue::Ed25519(Bytes64 {
            value: pair.sign(msg).0,
        })
    }
}

// Weight for Sr25519 sig verification
pub const SR25519_WEIGHT: Weight = Weight::from_ref_time(140_000_000);
// Weight for Ed25519 sig verification
pub const ED25519_WEIGHT: Weight = Weight::from_ref_time(152_000_000);

// XXX: Substrate UI can't parse them. Maybe later versions will fix it.
/*
/// Size of a Sr25519 public key in bytes.
pub const Sr25519_PK_BYTE_SIZE: usize = 32;
/// Size of a Ed25519 public key in bytes.
pub const Ed25519_PK_BYTE_SIZE: usize = 32;

#[derive(Encode, Decode, scale_info_derive::TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum PublicKey {
    Sr25519([u8; 32]),
    Ed25519([u8; 32])
}*/

/*#[derive(Encode, Decode, scale_info_derive::TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum PublicKey {
    Sr25519(Bytes32),
    Ed25519(Bytes32)
}
*/
