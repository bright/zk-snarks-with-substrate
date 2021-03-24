#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
		weights::Pays,
		sp_runtime::{
			MultiSignature, MultiSigner,
			traits::{CheckedAdd, Saturating, Zero},
		},
	};
	use frame_system::pallet_prelude::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The Balances of your system.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// The additional deposit needed to place a gift. Should be greater than the existential
		/// deposit to avoid killing the gifter account.
		type GiftDeposit: Get<BalanceOf<Self>>;
		/// The minimum gift amount. Should be greater than the existential deposit.
		type MinimumGift: Get<BalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode)]
	pub struct GiftInfo<AccountId, Balance> {
		gifter: AccountId,
		deposit: Balance,
		amount: Balance,
	}

	#[pallet::storage]
	#[pallet::getter(fn gifts)]
	pub type Gifts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		GiftInfo<T::AccountId, BalanceOf<T>>,
		OptionQuery
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A gift has been created! [amount, claimer]
		GiftCreated(BalanceOf<T>, T::AccountId),
		/// A gift has been claimed! [claimer, amount, to]
		GiftClaimed(T::AccountId, BalanceOf<T>, T::AccountId),
		/// A gift has been removed :( [to]
		GiftRemoved(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// User already has a pending gift.
		PendingGift,
		/// Don't be cheap... Get your friend a good gift!
		GiftTooSmall,
		/// An overflow has occurred.
		Overflow,
		/// A gift does not exist for this user.
		NoGift,
		/// You are not the owner of this gift.
		NotGifter,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(0)]
		fn gift(origin: OriginFor<T>, amount: BalanceOf<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(!Gifts::<T>::contains_key(&to), Error::<T>::PendingGift);
			ensure!(amount >= T::MinimumGift::get(), Error::<T>::GiftTooSmall);

			let deposit = T::GiftDeposit::get();
			let total_reserve = amount.checked_add(&deposit).ok_or(Error::<T>::Overflow)?;
			T::Currency::reserve(&who, total_reserve)?;

			// All checks have passed, so let's create the gift.

			let gift = GiftInfo {
				gifter: who,
				deposit,
				amount,
			};

			Gifts::<T>::insert(&to, gift);
			Self::deposit_event(Event::<T>::GiftCreated(amount, to));
			Ok(().into())
		}

		#[pallet::weight((0, Pays::No))] // Does not pay fee, so we MUST prevalidate this function
		fn claim(origin: OriginFor<T>, to: T::AccountId, _signature: MultiSignature) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// In the pre-validation function we confirmed that the signature is accurate,
			// so we don't need to validate again here and we can take the storage item directly.
			let gift = Gifts::<T>::take(&to).ok_or(Error::<T>::NoGift)?;

			let err_amount = T::Currency::unreserve(&gift.gifter, gift.deposit.saturating_add(gift.amount));
			// Should always have enough reserved unless there is a bug somewhere.
			debug_assert!(err_amount.is_zero());
			let res = T::Currency::transfer(&who, &to, gift.amount, ExistenceRequirement::AllowDeath);
			// Should never fail because we unreserve more than this above.
			debug_assert!(res.is_ok());

			Self::deposit_event(Event::<T>::GiftClaimed(who, gift.amount, to));
			Ok(().into())
		}

		#[pallet::weight(0)]
		fn remove(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let gift = Gifts::<T>::get(&to).ok_or(Error::<T>::NoGift)?;
			ensure!(gift.gifter == who, Error::<T>::NotGifter);

			let err_amount = T::Currency::unreserve(&gift.gifter, gift.deposit.saturating_add(gift.amount));
			// Should always have enough reserved unless there is a bug somewhere.
			debug_assert!(err_amount.is_zero());

			Gifts::<T>::remove(&to);

			Self::deposit_event(Event::<T>::GiftRemoved(to));
			Ok(().into())
		}
	}
}

// /// Validate `claim` calls prior to execution. Needed to avoid a DoS attack since they are
// /// otherwise free to place on chain.
// #[derive(Encode, Decode, Clone, Eq, PartialEq)]
// pub struct PrevalidateGiftClaim<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>) where
// 	<T as frame_system::Config>::Call: IsSubType<Call<T>>;

// impl<T: Config + Send + Sync> Debug for PrevalidateGiftClaim<T> where
// 	<T as frame_system::Config>::Call: IsSubType<Call<T>>
// {
// 	#[cfg(feature = "std")]
// 	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
// 		write!(f, "PrevalidateGiftClaim")
// 	}

// 	#[cfg(not(feature = "std"))]
// 	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
// 		Ok(())
// 	}
// }

// impl<T: Config + Send + Sync> PrevalidateGiftClaim<T> where
// 	<T as frame_system::Config>::Call: IsSubType<Call<T>>
// {
// 	/// Create new `SignedExtension` to check runtime version.
// 	pub fn new() -> Self {
// 		Self(sp_std::marker::PhantomData)
// 	}
// }

// impl<T: Config + Send + Sync> SignedExtension for PrevalidateGiftClaim<T> where
// 	<T as frame_system::Config>::Call: IsSubType<Call<T>>
// {
// 	type AccountId = T::AccountId;
// 	type Call = <T as frame_system::Config>::Call;
// 	type AdditionalSigned = ();
// 	type Pre = ();
// 	const IDENTIFIER: &'static str = "PrevalidateGiftClaim";

// 	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
// 		Ok(())
// 	}

// 	// <weight>
// 	// The weight of this logic is included in the `attest` dispatchable.
// 	// </weight>
// 	fn validate(
// 		&self,
// 		who: &Self::AccountId,
// 		call: &Self::Call,
// 		_info: &DispatchInfoOf<Self::Call>,
// 		_len: usize,
// 	) -> TransactionValidity {
// 		if let Some(local_call) = call.is_sub_type() {
// 			if let Call::claim(pub_key, signature) = local_call {

// 				let signer = Preclaims::<T>::get(who)
// 					.ok_or(InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()))?;
// 				if let Some(s) = Signing::get(signer) {
// 					let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
// 					ensure!(&attested_statement[..] == s.to_text(), e);
// 				}
// 			}
// 		}
// 		Ok(ValidTransaction::default())
// 	}
// }
