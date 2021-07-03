//! Benchmarking setup for pallet-kitties

use super::*;
use crate::{Substratekitties as PalletModule, *};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks! {
	sort_vector {
		let x in 0 .. 10000;
		let mut m = Vec::<u32>::new();
		for i in (0..x).rev() {
			m.push(i);
		}
	}: {
		// The benchmark execution phase could also be a closure with custom code
		m.sort();
	}
}

impl_benchmark_test_suite!(
	PalletModule,
	crate::tests::new_test_ext(),
	crate::tests::Test,
);
