//! Autogenerated weights for bbs_plus
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Interpreted, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/production/dock-node
// benchmark
// --execution=native
// --chain=mainnet
// --pallet=bbs_plus
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core/src/modules/bbs_plus/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for bbs_plus.
pub trait WeightInfo {
	fn add_params_sr25519(b: u32, l: u32) -> Weight;
	fn add_params_ed25519(b: u32, l: u32) -> Weight;
	fn add_params_secp256k1(b: u32, l: u32) -> Weight;
	fn remove_params_sr25519() -> Weight;
	fn remove_params_ed25519() -> Weight;
	fn remove_params_secp256k1() -> Weight;
	fn add_public_sr25519(b: u32) -> Weight;
	fn add_public_ed25519(b: u32) -> Weight;
	fn add_public_secp256k1(b: u32) -> Weight;
	fn remove_public_sr25519() -> Weight;
	fn remove_public_ed25519() -> Weight;
	fn remove_public_secp256k1() -> Weight;
}

/// Weights for bbs_plus using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_params_sr25519(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(52_181_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(7_000_u64).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(9_000_u64).saturating_mul(l as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	fn add_params_ed25519(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(52_658_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(l as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	fn add_params_secp256k1(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(154_268_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(1_000_u64).saturating_mul(b as u64))
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(2_000_u64).saturating_mul(l as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	fn remove_params_sr25519() -> Weight {
		Weight::from_ref_time(56_041_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn remove_params_ed25519() -> Weight {
		Weight::from_ref_time(52_544_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn remove_params_secp256k1() -> Weight {
		Weight::from_ref_time(155_224_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn add_public_sr25519(b: u32) -> Weight {
		Weight::from_ref_time(59_312_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(12_000_u64).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn add_public_ed25519(b: u32) -> Weight {
		Weight::from_ref_time(58_693_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(b as u64))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn add_public_secp256k1(_b: u32) -> Weight {
		Weight::from_ref_time(162_846_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn remove_public_sr25519() -> Weight {
		Weight::from_ref_time(59_284_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn remove_public_ed25519() -> Weight {
		Weight::from_ref_time(57_625_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn remove_public_secp256k1() -> Weight {
		Weight::from_ref_time(161_804_000_u64)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_params_sr25519(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(52_181_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(7_000_u64).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(9_000_u64).saturating_mul(l as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	fn add_params_ed25519(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(52_658_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(b as u64))
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(l as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	fn add_params_secp256k1(b: u32, l: u32) -> Weight {
		Weight::from_ref_time(154_268_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(1_000_u64).saturating_mul(b as u64))
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(2_000_u64).saturating_mul(l as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	fn remove_params_sr25519() -> Weight {
		Weight::from_ref_time(56_041_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn remove_params_ed25519() -> Weight {
		Weight::from_ref_time(52_544_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn remove_params_secp256k1() -> Weight {
		Weight::from_ref_time(155_224_000_u64)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn add_public_sr25519(b: u32) -> Weight {
		Weight::from_ref_time(59_312_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(12_000_u64).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn add_public_ed25519(b: u32) -> Weight {
		Weight::from_ref_time(58_693_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(3_000_u64).saturating_mul(b as u64))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn add_public_secp256k1(_b: u32) -> Weight {
		Weight::from_ref_time(162_846_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn remove_public_sr25519() -> Weight {
		Weight::from_ref_time(59_284_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn remove_public_ed25519() -> Weight {
		Weight::from_ref_time(57_625_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn remove_public_secp256k1() -> Weight {
		Weight::from_ref_time(161_804_000_u64)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}