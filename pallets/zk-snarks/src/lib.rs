#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;
pub use weights::*;

pub mod verifier;
pub use verifier::*;

use frame_support::storage::bounded_vec::BoundedVec;
pub use pallet::*;

type ProofDef<T> = BoundedVec<u8, <T as Config>::MaxProofLength>;
type VerificationKey<T> = BoundedVec<u8, <T as Config>::MaxVerificationKeyLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
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
		/// The verification key is to long.
		TooLongVerificationKey,
		/// The proof is too long.
		TooLongProof,
		/// The proof is too short.
		ProofIsEmpty,
		/// Verification key, not set.
		VerificationKeyIsNotSet,
	}

	/// Storing a public input.
	#[pallet::storage]
	pub type PublicInputStorage<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Storing a proof.
	#[pallet::storage]
	pub type ProofStorage<T: Config> = StorageValue<_, ProofDef<T>, ValueQuery>;

	/// Storing a verification key.
	#[pallet::storage]
	pub type VerificationKeyStorage<T: Config> = StorageValue<_, VerificationKey<T>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Store a verification key.
		#[pallet::weight(<T as Config>::WeightInfo::setup_verification_benchmark(vec_vk.len()))]
		pub fn setup_verification(
			_origin: OriginFor<T>,
			pub_input: u32,
			vec_vk: Vec<u8>,
		) -> DispatchResult {
			// Setting the public input data.
			PublicInputStorage::<T>::put(pub_input);

			// Setting the verification key.
			if vec_vk.is_empty() {
				VerificationKeyStorage::<T>::kill();
			} else {
				let vk: VerificationKey<T> =
					vec_vk.try_into().map_err(|_| Error::<T>::TooLongVerificationKey)?;
				VerificationKeyStorage::<T>::put(vk);
				Self::deposit_event(Event::<T>::VerificationSetupCompleted);
			}
			Ok(())
		}

		/// Verify a proof.
		#[pallet::weight(<T as Config>::WeightInfo::verify_benchmark(vec_proof.len()))]
		pub fn verify(_origin: OriginFor<T>, vec_proof: Vec<u8>) -> DispatchResult {            
            ensure!(!vec_proof.is_empty(), Error::<T>::ProofIsEmpty);

            let proof: ProofDef<T> = vec_proof.try_into().map_err(|_| Error::<T>::TooLongProof)?;
            ProofStorage::<T>::put(proof.clone());
            Self::deposit_event(Event::<T>::VerificationProofSet);

            let v = Verifier { key: <VerificationKeyStorage<T>>::get().clone().into_inner() };
            if v.verify_proof(PublicInputStorage::<T>::get().clone(), proof.into_inner())
                .map_err(|_| Error::<T>::VerificationKeyIsNotSet)?
            {
                Self::deposit_event(Event::<T>::VerificationSuccess);
            } else {
                Self::deposit_event(Event::<T>::VerificationFailed);
            }
			Ok(())
		}
	}
}
