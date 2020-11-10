#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use sp_std::prelude::*;
use frame_system::ensure_signed;
use sp_runtime::{DispatchResult, traits::Dispatchable};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, Parameter,
	weights::{Pays, GetDispatchInfo},
	dispatch::DispatchResultWithPostInfo,
	traits::Get,
};

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// The call type from the runtime which has all the calls available in your runtime.
	type Call: Parameter + GetDispatchInfo + Dispatchable<Origin=Self::Origin>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Feeless {
		// Track how many calls each user has done for the latest session
		Tracker: map hasher(twox_64_concat) T::AccountId => (T::BlockNumber, u32);
		// Max calls to be made per session
		MaxCalls: u32 = 100;
		// Length of a session
		SessionLength: T::BlockNumber = 1000.into();
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		ExtrinsicResult(AccountId, DispatchResult),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		// User has used all of their free calls.
		NoFreeCalls,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = {
			let dispatch_info = call.get_dispatch_info();
			(
				dispatch_info.weight.saturating_add(T::DbWeight::get().reads_writes(3, 1)),
				dispatch_info.class,
				dispatch_info.pays_fee
			)
		}]
		fn make_feeless(origin, call: Box<<T as Trait>::Call>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin.clone())?;

			// Get the relevant storage data.
			let max_calls = MaxCalls::get();
			let (last_user_session, mut user_calls) = Tracker::<T>::get(&sender);
			let current_block_number = frame_system::Module::<T>::block_number();
			let session_length = SessionLength::<T>::get();

			// Calculate the current session.
			let current_session = current_block_number / session_length;

			// If this is a new session for the user, reset their count.
			if last_user_session < current_session {
				user_calls = 0;
			}

			// Check that the user has an available free call
			if user_calls < max_calls {
				// Update the tracker count.
				Tracker::<T>::insert(
					&sender,
					(
						current_session,
						user_calls.saturating_add(1),
					)
				);

				// Dispatch the call
				let result = call.dispatch(origin);

				// Deposit an event with the result
				Self::deposit_event(
					RawEvent::ExtrinsicResult(
						sender,
						result.map(|_| ()).map_err(|e| e.error),
					)
				);

				// Make the tx feeless!
				return Ok(Pays::No.into())
			} else {
				// They do not have enough feeless txs, so we charge them
				// for the reads.
				//
				// Note: This could be moved into a signed extension check to
				// avoid charging them any fees at all in any situation.
				let check_logic_weight = T::DbWeight::get().reads(3);
				// Return the reduced weight
				return Ok(Some(check_logic_weight).into())
			}
		}
	}
}
