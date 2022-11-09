
//! Autogenerated weights for `pallet_zk_snarks`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-18, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Michals-MacBook-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/node-template
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet-zk-snarks
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// /Users/mg/Development/substrate/groth16-verifier/pallets/zk-snarks/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_example_basic.
pub trait WeightInfo {
	fn setup_verification_benchmark(len: usize,) -> Weight;
	fn verify_benchmark(len: usize,) -> Weight;
}

/// Weight functions for `pallet_zk_snarks`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: ZKSnarks VerificationKeyStorage (r:0 w:1)
	fn setup_verification_benchmark(len: usize,) -> Weight {
		// Minimum execution time: 21_000 nanoseconds.
		Weight::from_ref_time(22_000_000 as u64).saturating_mul(len as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: ZKSnarks VerificationKeyStorage (r:1 w:0)
	// Storage: ZKSnarks ProofStorage (r:0 w:1)
	fn verify_benchmark(len: usize,) -> Weight {
		// Minimum execution time: 31_000 nanoseconds.
		Weight::from_ref_time(32_000_000 as u64).saturating_mul(len as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn setup_verification_benchmark(_len: usize,) -> Weight {
        Weight::zero()
    }

	fn verify_benchmark(_len: usize,) -> Weight {
        Weight::zero()
    }
}
