#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

extern crate alloc;

pub mod common;
pub mod modules;
pub mod util;

pub use modules::{
	accumulator, anchor, attest, blob, did, offchain_signatures, revoke, status_list_credential,
	trusted_entity,
};

#[cfg(test)]
mod tests;
