use super::*;
use frame_benchmarking::benchmarks;
use frame_support::traits::Get;
use frame_system::RawOrigin;

use crate::Pallet as ZKSnarks;

benchmarks! {
	setup_verification_benchmark {
		let mut key = Vec::<u8>::new();
		let max = T::MaxVerificationKeyLength::get();
		for i in 0..max {
			key.push(i as u8);
		}
	}: setup_verification(RawOrigin::None, 50, key)

	verify_benchmark {
		let mut proof = Vec::<u8>::new();
		for i in 0..T::MaxProofLength::get() {
			proof.push(i as u8);
		}
		ZKSnarks::<T>::setup_verification(RawOrigin::None.into(), 50, proof.clone()).expect("This should work...");
	}: verify(RawOrigin::None, proof)

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test)
}
