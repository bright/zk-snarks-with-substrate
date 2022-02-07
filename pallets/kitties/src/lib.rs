#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.io/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		traits::{ Randomness, Currency, tokens::ExistenceRequirement },
	};
	use sp_io::hashing::blake2_128;
	use scale_info::TypeInfo;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],   			 // Using 16 bytes to represent a kitty DNA
		pub price: Option<BalanceOf<T>>, // None, assumes not for sale 
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

	// Set Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// The type of Randomness we want to specify for this pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// Handles arithemtic overflow when incrementing the Kitty counter.
		KittyCntOverflow,
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		NonExistantKitty,
		/// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty. 
		NotEnoughBalance,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Kitty was sucessfully created. \[sender, kitty_dna\]
		Created(T::AccountId, [u8; 16]),
		/// Kitty price was sucessfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, [u8; 16], Option<BalanceOf<T>>),
		/// A Kitty was sucessfully transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, [u8; 16]),
		/// A Kitty was sucessfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, [u8; 16], BalanceOf<T>),
	}

	// Storage items.

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	/// Keeps track of the number of Kitties in existence.
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Maps the Kitty struct to the Kitty DNA.
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	/// Tracks the maximum allowed Kitties an account can own. 
	pub(super) type KittiesOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<[u8; 16], T::MaxKittyOwned>, ValueQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be supplied.
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, dna.clone(), gender.clone());
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
			let sender = ensure_signed(origin)?;

			// Generate unique DNA and Gender.
			let (kitty_gen_dna, gender) = Self::gen_dna();

			// Write new Kitty to storage.
			let kitty_dna = Self::mint(&sender, kitty_gen_dna, gender)?;

			// Logging to the console
			log::info!("🎈😺 A kitty is born with ID ➡ {:?}.", kitty_dna);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created(sender, kitty_dna));
			Ok(())
		}

		/// Breed a Kitty.
		///
		/// Breed two Kitties to give birth to a new Kitty.
		#[pallet::weight(0)]
		pub fn breed_kitty(
			origin: OriginFor<T>, 
			parent_1: [u8; 16], 
			parent_2: [u8; 16]
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Check both parents are owner by the caller of this function.
			ensure!(Self::check_owner(&parent_1, &sender), <Error<T>>::NotKittyOwner);
			ensure!(Self::check_owner(&parent_2, &sender), <Error<T>>::NotKittyOwner);

			// Create new DNA from these parents.
			let (new_dna, new_gender) = Self::breed_dna(&parent_1, &parent_2);

			// Mint new Kitty.
			Self::mint(&sender, new_dna, new_gender)?;
			Ok(())
		}

		/// Directly transfer a kitty to another recipient.
		///
		/// Any account that holds a kitty can send it to another Account. This will reset the asking
		/// price of the kitty, marking it not for sale.
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>, 
			to: T::AccountId, 
			kitty_id: [u8; 16]
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::check_owner(&kitty_id, &from), <Error<T>>::NotKittyOwner);

			// Verify the kitty is not transferring back to its owner.
			ensure!(from != to, <Error<T>>::TransferToSelf);

			// Verify the recipient has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

			Self::transfer_kitty_to(&kitty_id, &to)?;

			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(())
		}

		/// Buy a saleable Kitty. The bid price provided from the buyer has to be equal or higher
		/// than the ask price from the seller.
		///
		/// This will reset the asking price of the kitty, marking it not for sale.
		/// Marking this method `transactional` so when an error is returned, we ensure no storage is changed.
		#[pallet::weight(0)]
		pub fn buy_kitty(
			origin: OriginFor<T>, 
			kitty_id: [u8; 16], 
			bid_price: BalanceOf<T>
		) -> DispatchResult {

			let buyer = ensure_signed(origin)?;

			// Check the kitty exists and buyer is not the current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::NonExistantKitty)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			// Check the kitty is for sale and the kitty ask price <= bid_price
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}

			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);

			// Verify the buyer has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

			let seller = kitty.owner.clone();

			// Transfer the amount from buyer to seller
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the kitty from seller to buyer
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));

			Ok(())
		}

		/// Set the price for a Kitty.
		///
		/// Updates Kitty price and updates storage.
		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>, 
			kitty_id: [u8; 16], 
			new_price: Option<BalanceOf<T>>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::check_owner(&kitty_id, &sender), <Error<T>>::NotKittyOwner);

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::NonExistantKitty)?;

			kitty.price = new_price.clone();
			<Kitties<T>>::insert(&kitty_id, kitty);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(())
		}
	}


	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {

		// Generates and returns DNA and Gender
		fn gen_dna() -> ([u8; 16], Gender) {
			
			// Create randomness
			let random = T::KittyRandomness::random(&b"dna"[..]).0;

			// Create randomness payload
			let payload = (
				random,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);

			if random.as_ref()[0] % 2 == 0 {
				return (payload.using_encoded(blake2_128), Gender::Male)

			} else { 
				return (payload.using_encoded(blake2_128), Gender::Female)
			}
		}

		// Picks from existing DNA.
		fn mutate_dna_fragment(dna_fragment1: u8, dna_fragment2: u8, new_dna_fragment: u8) -> u8 {
			if new_dna_fragment % 2 == 0 {
				dna_fragment1
			} else {
				dna_fragment2
			}
		}

		// Generates a new Kitty using existing Kitties.
		pub fn breed_dna(parent1: &[u8; 16], parent2: &[u8; 16]) -> ([u8; 16], Gender) {

			let (mut new_dna, new_gender) = Self::gen_dna();

			for i in 0..new_dna.len() {
				new_dna[i] = Self::mutate_dna_fragment(parent1[i], parent2[1], new_dna[i])
			}
			return (new_dna, new_gender)
		}

		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: [u8; 16],
			gender: Gender,
		) -> Result<[u8; 16], Error<T>> {

			// Create a new object.
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				price: None,
				gender: gender.clone(),
				owner: owner.clone(),
			};

			// Check if the kitty does not already exist in our storage map
			ensure!(Self::kitties(&kitty.dna) == None, Error::<T>::NonExistantKitty);

			// Performs this operation first as it may fail
			let new_cnt = Self::kitty_cnt().checked_add(1)
				.ok_or(Error::<T>::KittyCntOverflow)?;

			// Performs this operation first as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| {
				kitty_vec.try_push(kitty.dna)
			}).map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			// Write new Kitty to storage.
			Kitties::<T>::insert(kitty.dna, kitty);
			KittyCnt::<T>::put(new_cnt);
			Ok(dna)
		}

		// Check whether Kitty is owner by the breeder.
		pub fn check_owner(kitty_dna: &[u8; 16], breeder: &T::AccountId) -> bool {
			match Self::kitties(kitty_dna) {
				Some(kitty) => kitty.owner == *breeder, 
				None => false
			}
		}

		// Update storage to transfer kitty.
		pub fn transfer_kitty_to(
			kitty_id: &[u8; 16],
			to: &T::AccountId,
		) -> Result<(), Error<T>> {

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::NonExistantKitty)?;

			let prev_owner = kitty.owner.clone();

			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::NonExistantKitty)?;

			// Update the kitty owner
			kitty.owner = to.clone();
			// Reset the ask price so the kitty is not for sale untill `set_price()` is called
			// by the current owner.
			kitty.price = None;

			<Kitties<T>>::insert(kitty_id, kitty);

			<KittiesOwned<T>>::try_mutate(to, |vec| {
				vec.try_push(*kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
