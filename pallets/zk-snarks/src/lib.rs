// MIT License

// Copyright (c) 2022 Bright Inventions

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.


#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate uint;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;
pub use weights::*;

mod deserialization;
pub mod verify;

use frame_support::storage::bounded_vec::BoundedVec;
pub use pallet::*;
use sp_std::vec::Vec;

type PublicInputsDef<T> = BoundedVec<u8, <T as Config>::MaxPublicInputsLength>;
type ProofDef<T> = BoundedVec<u8, <T as Config>::MaxProofLength>;
type VerificationKeyDef<T> = BoundedVec<u8, <T as Config>::MaxVerificationKeyLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{
		deserialization::{deserialize_public_inputs, Proof, VKey},
		verify::{SUPPORTED_CURVE, SUPPORTED_PROTOCOL},
	};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxPublicInputsLength: Get<u32>;

		/// The maximum length of the proof.
		#[pallet::constant]
		type MaxProofLength: Get<u32>;

		/// The maximum length of the verification key.
		#[pallet::constant]
		type MaxVerificationKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VerificationSetupCompleted,
		VerificationProofSet,
		VerificationSuccess,
		VerificationFailed,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Public inputs vector is to long.
		TooLongPublicInputs,
		/// The verification key is to long.
		TooLongVerificationKey,
		/// The proof is too long.
		TooLongProof,
		/// The proof is too short.
		ProofIsEmpty,
		/// Verification key, not set.
		VerificationKeyIsNotSet,
		/// Malformed key
		MalformedVerificationKey,
		/// Malformed proof
		MalformedProof,
		/// Curve is not supported
		NotSupportedCurve,
		/// Protocol is not supported
		NotSupportedProtocol,
	}

	/// Storing a public input.
	#[pallet::storage]
	pub type PublicInputStorage<T: Config> = StorageValue<_, PublicInputsDef<T>, ValueQuery>;

	/// Storing a proof.
	#[pallet::storage]
	pub type ProofStorage<T: Config> = StorageValue<_, ProofDef<T>, ValueQuery>;

	/// Storing a verification key.
	#[pallet::storage]
	pub type VerificationKeyStorage<T: Config> = StorageValue<_, VerificationKeyDef<T>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Store a verification key.
		#[pallet::weight(<T as Config>::WeightInfo::setup_verification_benchmark(vec_vk.len()))]
		pub fn setup_verification(
			_origin: OriginFor<T>,
			pub_input: Vec<u8>,
			vec_vk: Vec<u8>,
		) -> DispatchResult {
			// Setting the public input data.
			let public_inputs: PublicInputsDef<T> =
				pub_input.try_into().map_err(|_| Error::<T>::TooLongPublicInputs)?;
			let _deserialized_public_inputs = deserialize_public_inputs(public_inputs.as_slice());
			PublicInputStorage::<T>::put(public_inputs);
			let vk: VerificationKeyDef<T> =
				vec_vk.try_into().map_err(|_| Error::<T>::TooLongVerificationKey)?;
			let deserialized_vk = VKey::from_json_u8_slice(vk.as_slice())
				.map_err(|_| Error::<T>::MalformedVerificationKey)?;
			ensure!(
				deserialized_vk.curve == SUPPORTED_CURVE.as_bytes(),
				Error::<T>::NotSupportedCurve
			);
			ensure!(
				deserialized_vk.protocol == SUPPORTED_PROTOCOL.as_bytes(),
				Error::<T>::NotSupportedProtocol
			);
			VerificationKeyStorage::<T>::put(vk);
			Self::deposit_event(Event::<T>::VerificationSetupCompleted);
			Ok(())
		}

		/// Verify a proof.
		#[pallet::weight(<T as Config>::WeightInfo::verify_benchmark(vec_proof.len()))]
		pub fn verify(_origin: OriginFor<T>, vec_proof: Vec<u8>) -> DispatchResult {
			ensure!(!vec_proof.is_empty(), Error::<T>::ProofIsEmpty);
			let proof: ProofDef<T> = vec_proof.try_into().map_err(|_| Error::<T>::TooLongProof)?;
			let deserialized_proof = Proof::from_json_u8_slice(proof.as_slice())
				.map_err(|_| Error::<T>::MalformedProof)?;
			ensure!(
				deserialized_proof.curve == SUPPORTED_CURVE.as_bytes(),
				Error::<T>::NotSupportedCurve
			);
			ensure!(
				deserialized_proof.protocol == SUPPORTED_PROTOCOL.as_bytes(),
				Error::<T>::NotSupportedProtocol
			);
			ProofStorage::<T>::put(proof.clone());
			Self::deposit_event(Event::<T>::VerificationProofSet);
			Self::deposit_event(Event::<T>::VerificationSuccess);
			Ok(())
		}
	}
}
