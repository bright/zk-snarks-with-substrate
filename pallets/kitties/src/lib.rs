#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{tokens::ExistenceRequirement, Currency, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Handles our pallet's currency abstraction
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding kitty information
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		// Using 16 bytes to represent a kitty DNA
		pub dna: [u8; 16],
		// `None` assumes not for sale
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: T::AccountId,
	}

	// Set Gender type in kitty struct
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	// We need this to pass kitty info for genesis configuration
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the kitties pallet.
		type Currency: Currency<Self::AccountId>;

		/// The maximum amount of kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// The type of Randomness we want to specify for this pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors
	#[pallet::error]
	pub enum Error<T> {
		/// Handles arithemtic overflow when incrementing the kitty counter.
		CountForKittyOverflow,
		/// An account cannot own more kitties than `MaxCountForKitty`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the kitty exists.
		NonExistantKitty,
		/// Handles checking that the kitty is owned by the account transferring, buying or setting
		/// a price for it.
		NotKittyOwner,
		/// Ensures the kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a kitty.
		NotEnoughBalance,
		/// Owner can't use two kitties of the same genfer to breed.
		ThoseCatsCantBreed,
	}

	// Events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new kitty was successfully created.
		Created { kitty: [u8; 16], owner: T::AccountId },
		/// The price of a kitty was successfully set.
		PriceSet { kitty: [u8; 16], price: Option<BalanceOf<T>> },
		/// A kitty was successfully transferred.
		Transferred { from: T::AccountId, to: T::AccountId, kitty: [u8; 16] },
		/// A kitty was successfully bought.
		Bought { buyer: T::AccountId, seller: T::AccountId, kitty: [u8; 16], price: BalanceOf<T> },
	}

	// Storage items

	/// Keeps track of the number of kitties in existence.
	#[pallet::storage]
	#[pallet::getter(fn kitty_count)]
	pub(super) type CountForKitty<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the kitty struct to the kitty DNA.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Kitty<T>>;

	/// Track the kitties owned by each account.
	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<[u8; 16], T::MaxKittyOwned>,
		ValueQuery,
	>;

	// Our pallet's genesis configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	// Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the DNA and Gender to be
			// supplied
			for (account, dna, gender) in &self.kitties {
				let _ = Pallet::<T>::mint(account, dna.clone(), gender.clone());
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual kitty creation is done in the `mint()` function.
		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Generate unique DNA and Gender using a helper function
			let (kitty_gen_dna, gender) = Self::gen_dna();

			// Write new kitty to storage by calling helper function
			let kitty_dna = Self::mint(&sender, kitty_gen_dna, gender)?;

			// Logging to the console
			log::info!("🎈😺 A kitty is born with ID ➡ {:?}.", kitty_dna);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created { kitty: kitty_dna, owner: sender });
			Ok(())
		}

		/// Breed a kitty.
		///
		/// Breed two kitties to give birth to a new kitty.
		#[pallet::weight(0)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent_1: [u8; 16],
			parent_2: [u8; 16],
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Check both parents are owned by the caller of this function
			ensure!(Self::check_owner(&parent_1, &sender), Error::<T>::NotKittyOwner);
			ensure!(Self::check_owner(&parent_2, &sender), Error::<T>::NotKittyOwner);

			// Parents must be of opposite genders
			let maybe_mom = Self::kitties(&parent_1).ok_or(Error::<T>::NonExistantKitty)?;
			let maybe_dad = Self::kitties(&parent_2).ok_or(Error::<T>::NonExistantKitty)?;

			ensure!(maybe_mom.gender != maybe_dad.gender, Error::<T>::ThoseCatsCantBreed);

			// Create new DNA from these parents
			let (new_dna, new_gender) = Self::breed_dna(&parent_1, &parent_2);

			// Mint new kitty
			Self::mint(&sender, new_dna, new_gender)?;
			Ok(())
		}

		/// Directly transfer a kitty to another recipient.
		///
		/// Any account that holds a kitty can send it to another Account. This will reset the
		/// asking price of the kitty, marking it not for sale.
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: [u8; 16],
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::check_owner(&kitty_id, &from), Error::<T>::NotKittyOwner);

			// Verify the kitty is not transferring back to its owner.
			ensure!(from != to, Error::<T>::TransferToSelf);

			// Verify the recipient has the capacity to receive one more kitty
			let to_owned = KittiesOwned::<T>::decode_len(&to).unwrap_or(0);
			ensure!((to_owned as u32) < T::MaxKittyOwned::get(), Error::<T>::ExceedMaxKittyOwned);

			// Write to storage using helper function
			Self::transfer_kitty_to(&kitty_id, &to)?;

			// Deposit an event
			Self::deposit_event(Event::Transferred { from, to, kitty: kitty_id });

			Ok(())
		}

		/// Buy a saleable kitty. The bid price provided from the buyer has to be equal or higher
		/// than the ask price from the seller.
		///
		/// This will reset the asking price of the kitty, marking it not for sale.
		/// Marking this method `transactional` so when an error is returned, we ensure no storage
		/// is changed.
		#[pallet::weight(0)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: [u8; 16],
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let buyer = ensure_signed(origin)?;

			// Check the kitty exists and buyer is not the current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::NonExistantKitty)?;
			ensure!(kitty.owner != buyer, Error::<T>::BuyerIsKittyOwner);

			// First check if the kitty is for sale
			let current_price = kitty.price;
			ensure!(current_price != None, Error::<T>::KittyNotForSale);
			// Then ensure that the bid_price > asking price
			ensure!(current_price <= Some(bid_price), Error::<T>::KittyBidPriceTooLow);

			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, Error::<T>::NotEnoughBalance);

			// Verify the recipient has the capacity to receive one more kitty
			let to_owned = KittiesOwned::<T>::decode_len(&buyer).unwrap_or(0);
			ensure!((to_owned as u32) < T::MaxKittyOwned::get(), Error::<T>::ExceedMaxKittyOwned);

			// Transfer the amount from buyer to seller
			let seller = kitty.owner.clone();
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the kitty from seller to buyer
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			// Deposit an event
			Self::deposit_event(Event::Bought { buyer, seller, kitty: kitty_id, price: bid_price });

			Ok(())
		}

		/// Set the price for a kitty.
		///
		/// Updates kitty price and updates storage.
		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: [u8; 16],
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::check_owner(&kitty_id, &sender), Error::<T>::NotKittyOwner);

			// Get the kitty
			let mut kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::NonExistantKitty)?;

			// Set the price in storage
			kitty.price = new_price.clone();
			Kitties::<T>::insert(&kitty_id, kitty);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet { kitty: kitty_id, price: new_price });

			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		// Generates and returns DNA and Gender
		fn gen_dna() -> ([u8; 16], Gender) {
			// Create randomness
			let random = T::KittyRandomness::random(&b"dna"[..]).0;

			// Create randomness payload. Multiple kitties can be generated in the same block,
			// retaining uniqueness.
			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			// Turns into a byte array
			let encoded_payload = unique_payload.encode();
			let hash = blake2_128(&encoded_payload);

			// Generate Gender
			if hash[0] % 2 == 0 {
				return (hash, Gender::Male)
			} else {
				return (hash, Gender::Female)
			}
		}

		// Picks from existing DNA
		fn mutate_dna_fragment(dna_fragment1: u8, dna_fragment2: u8, new_dna_fragment: u8) -> u8 {
			if new_dna_fragment % 2 == 0 {
				dna_fragment1
			} else {
				dna_fragment2
			}
		}

		// Generates a new kitty using existing kitties
		pub fn breed_dna(parent1: &[u8; 16], parent2: &[u8; 16]) -> ([u8; 16], Gender) {
			let (mut new_dna, new_gender) = Self::gen_dna();

			for i in 0..new_dna.len() {
				new_dna[i] = Self::mutate_dna_fragment(parent1[i], parent2[1], new_dna[i])
			}
			return (new_dna, new_gender)
		}

		// Helper to mint a kitty
		pub fn mint(
			owner: &T::AccountId,
			dna: [u8; 16],
			gender: Gender,
		) -> Result<[u8; 16], Error<T>> {
			// Create a new object
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				price: None,
				gender: gender.clone(),
				owner: owner.clone(),
			};

			// Check if the kitty does not already exist in our storage map
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::NonExistantKitty);

			// Performs this operation first as it may fail
			let new_count =
				Self::kitty_count().checked_add(1).ok_or(Error::<T>::CountForKittyOverflow)?;

			// Append kitty to KittiesOwned
			KittiesOwned::<T>::try_append(&owner, kitty.dna)
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			// Write new kitty to storage
			Kitties::<T>::insert(kitty.dna, kitty);
			CountForKitty::<T>::put(new_count);

			// Returns the DNA of the new kitty if this suceeds
			Ok(dna)
		}

		// Check whether kitty is owner by the breeder
		pub fn check_owner(kitty_dna: &[u8; 16], breeder: &T::AccountId) -> bool {
			match Self::kitties(kitty_dna) {
				Some(kitty) => kitty.owner == *breeder,
				None => false,
			}
		}

		// Update storage to transfer kitty
		pub fn transfer_kitty_to(kitty_id: &[u8; 16], to: &T::AccountId) -> Result<(), Error<T>> {
			// Get the kitty
			let mut kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::NonExistantKitty)?;

			// Remove `kitty_id` from the KittyOwned vector
			let prev_owner = kitty.owner.clone();
			KittiesOwned::<T>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| Error::<T>::NonExistantKitty)?;

			// Verify (again!) that the recipient has the capacity to receive one more kitty
			let to_owned = KittiesOwned::<T>::decode_len(&to).unwrap_or(0);
			ensure!((to_owned as u32) < T::MaxKittyOwned::get(), Error::<T>::ExceedMaxKittyOwned);

			// Update the kitty owner and reset the price to `None`
			kitty.owner = to.clone();
			kitty.price = None;

			// Write updates to storage
			Kitties::<T>::insert(kitty_id, kitty);
			KittiesOwned::<T>::try_mutate(to, |vec| vec.try_push(*kitty_id))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
