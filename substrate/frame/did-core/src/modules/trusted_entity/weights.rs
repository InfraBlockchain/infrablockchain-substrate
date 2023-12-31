//! Autogenerated weights for add_issuer
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Interpreted, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/production/dock-node
// benchmark
// --execution=native
// --chain=mainnet
// --pallet=add_issuer
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core/src/modules/add_issuer/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for add_issuer.
pub trait WeightInfo {
	fn add_issuer_sr25519(r: u32) -> Weight;
	fn add_issuer_ed25519(r: u32) -> Weight;
	fn add_issuer_secp256k1(r: u32) -> Weight;
	fn remove_issuer_sr25519(r: u32) -> Weight;
	fn remove_issuer_ed25519(r: u32) -> Weight;
	fn remove_issuer_secp256k1(r: u32) -> Weight;
	fn add_verifier_sr25519(r: u32) -> Weight;
	fn add_verifier_ed25519(r: u32) -> Weight;
	fn add_verifier_secp256k1(r: u32) -> Weight;
	fn remove_verifier_sr25519(r: u32) -> Weight;
	fn remove_verifier_ed25519(r: u32) -> Weight;
	fn remove_verifier_secp256k1(r: u32) -> Weight;
	fn remove_authorizer_sr25519() -> Weight;
	fn remove_authorizer_ed25519() -> Weight;
	fn remove_authorizer_secp256k1() -> Weight;
	fn new_authorizer(c: u32) -> Weight;
}

/// Weights for add_issuer using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_issuer_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(51_886_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(744_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_issuer_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(55_942_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(718_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_issuer_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(148_000_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(707_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(67_695_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(741_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(65_882_000_u64)
			// Standard Error: 3_000
			.saturating_add(Weight::from_ref_time(747_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(166_568_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(704_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(51_886_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(744_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(55_942_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(718_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(148_000_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(707_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(67_695_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(741_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(65_882_000_u64)
			// Standard Error: 3_000
			.saturating_add(Weight::from_ref_time(747_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(166_568_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(704_000_u64).saturating_mul(r as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_authorizer_sr25519() -> Weight {
		Weight::from_ref_time(128_526_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(103_u64))
	}
	fn remove_authorizer_ed25519() -> Weight {
		Weight::from_ref_time(122_116_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(103_u64))
	}
	fn remove_authorizer_secp256k1() -> Weight {
		Weight::from_ref_time(230_576_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(103_u64))
	}
	fn new_authorizer(c: u32) -> Weight {
		Weight::from_ref_time(9_069_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(35_000_u64).saturating_mul(c as u64))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_issuer_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(51_886_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(744_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_issuer_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(55_942_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(718_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_issuer_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(148_000_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(707_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(67_695_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(741_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(65_882_000_u64)
			// Standard Error: 3_000
			.saturating_add(Weight::from_ref_time(747_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_issuer_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(166_568_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(704_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(51_886_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(744_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(55_942_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(718_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn add_verifier_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(148_000_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(707_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_sr25519(r: u32) -> Weight {
		Weight::from_ref_time(67_695_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(741_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_ed25519(r: u32) -> Weight {
		Weight::from_ref_time(65_882_000_u64)
			// Standard Error: 3_000
			.saturating_add(Weight::from_ref_time(747_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_verifier_secp256k1(r: u32) -> Weight {
		Weight::from_ref_time(166_568_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(704_000_u64).saturating_mul(r as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64.saturating_mul(r as u64)))
	}
	fn remove_authorizer_sr25519() -> Weight {
		Weight::from_ref_time(128_526_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(103_u64))
	}
	fn remove_authorizer_ed25519() -> Weight {
		Weight::from_ref_time(122_116_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(103_u64))
	}
	fn remove_authorizer_secp256k1() -> Weight {
		Weight::from_ref_time(230_576_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(103_u64))
	}
	fn new_authorizer(c: u32) -> Weight {
		Weight::from_ref_time(9_069_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(35_000_u64).saturating_mul(c as u64))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
