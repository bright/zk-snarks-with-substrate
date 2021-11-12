#![cfg_attr(not(feature = "std"), no_std)]

// Participation protocol:
// - `become_reviewer` - Stake amount to get voted in to become on the review committe.
// - `vote_for_reviewer` - Hunters can vote on reviewers they submitted to as part of the governance.
// - 

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type BalanceOf<T> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        
        // Handles bids for Society pallet
        type EnterSociety: Get<BalanceOf<Self>;
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		
		BidSuccessful(T::AccountId, T::BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn become_reviewer(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {

            let candidate = ensurge_signed(origin)?;

            T::EnterSociety::bid(candidate, amount)

            Self::deposit_event(Event::BidSuccessful(candidate, amount));

			Ok(())
		}

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote_for_reviewer(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {

            let candidate = ensurge_signed(origin)?;

            T::EnterSociety::bid(candidate, amount)
            
            Self::deposit_event(Event::BidSuccessful(candidate, amount));

			Ok(())
		}
	}
}
