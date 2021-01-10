// Copyright 2020-2021 Parity Technologies (UK) Ltd.
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use codec::{Decode, Encode};
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error,
	dispatch::{DispatchError, DispatchResult, Vec},
	ensure,
};
use frame_support::weights::Pays;
use frame_system::ensure_signed;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::collections::btree_map::BTreeMap;

#[cfg(test)]

mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
	/// The ubiquitous Event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub type Value = u128;
pub type UtxoHash = H256;
pub type TxHash = H256;

// TODO: Make these configurable?
const MAX_INPUTS: usize = 100;
const MAX_OUTPUTS: usize = 100;

/// Single transaction output to create upon transaction dispatch
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct TransactionOutput<AccountId> {
	/// Value associated with this output
	pub value: Value,

	/// Public key associated with this output. In order to spend this output
	/// owner must provide a proof by hashing the whole `Transaction` and
	/// signing it with a corresponding private key.
	pub pubkey: AccountId,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Transaction<AccountId> {
	/// UTXOs to be used as input for the transaction pair
	pub input: Vec<UtxoHash>,
	/// UTXOs to be created as a result of the transaction pair
	pub output: Vec<TransactionOutput<AccountId>>,
}

decl_storage! {
	trait Store for Module<T: Trait> as Utxo {
		/// All valid unspent transaction outputs are stored in this map.
		/// Initial set of UTXO is populated from the list stored in genesis.
		/// We use the identity hasher here because the cryptographic hashing is
		/// done explicitly.
		UtxoStore get(fn utxo) build(|config: &GenesisConfig<T>| {
			config.genesis_utxos
				.iter()
				.cloned()
				.map(|u| (BlakeTwo256::hash_of(&u), u))
				.collect::<Vec<_>>()
		}): map hasher(identity) UtxoHash => Option<TransactionOutput<T::AccountId>>;

		/// Maps an account id to its UTXOs.
		OwnedUtxos get(fn utxos_for): map hasher(blake2_128_concat) T::AccountId => Option<Vec<UtxoHash>>;
	}

	add_extra_genesis {
		config(genesis_utxos): Vec<TransactionOutput<T::AccountId>>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Given input for transaction is missing.
		InputUtxoMissing,
		/// Output should not be zero.
		OutputIsZero,
		/// An output already exists.
		OutputAlreadyExists,
		/// A calculation overflowed.
		Overflow,
		/// Output value should not exceed input value.
		OutputGreaterThanInput,
		/// Transactions should not be empty.
		EmptyTransaction,
		/// The transaction contains too many pairs.
		TransactionTooBig,
		/// A pair does not have input UTXOs.
		EmptyInput,
		/// A pair has too many input UTXOs.
		TooManyInputs,
		/// A pair does not have any outputs.
		EmptyOutput,
		/// A pair has too many outputs.
		TooManyOutputs,
		/// The transaction contains a duplicate input.
		DuplicateInput,
		/// The sender of the transaction is not the owner of one of the input UTXOs.
		NotOwner,
		/// Merging UTXOs is disallowed for regular transactions.
		MergingDisallowed,
	}
}

// External functions: callable by the end user
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Transfer the given UTXOs from the caller to account `to`.
		///
		/// Note: Will not merge UTXOs but transfer all of them as a bundle instead.
		#[weight = (0, Pays::No)]
		pub fn transfer(origin, utxos: Vec<UtxoHash>, to: T::AccountId) {
			let who = ensure_signed(origin)?;

			let transaction = utxos.into_iter().map(|hash| -> Result<Transaction<T::AccountId>, DispatchError> {
				let utxo = Self::utxo(hash).ok_or(Error::<T>::InputUtxoMissing)?;

				let mut output_utxo = utxo;
				output_utxo.pubkey = to.clone();
				Ok(Transaction {
					input: vec![hash],
					output: vec![output_utxo],
				})
			}).collect::<Result<Vec<Transaction<T::AccountId>>, DispatchError>>()?;
			Self::validate_signed(&transaction, Some(who))?;
			Self::update_storage(&transaction);
			Self::deposit_event(RawEvent::TransferSuccess(transaction))
		}

		/// Transfer UTXOs from the calling account according to the signed pairs.
		///
		/// Note: Disallows merging UTXOs (transaction pairs with more than one input).
		#[weight = (0, Pays::No)]
		pub fn send_transaction(origin, transaction: Vec<Transaction<T::AccountId>>) {
			let who = ensure_signed(origin)?;
			ensure!(transaction.iter().all(|pair| pair.input.len() == 1), Error::<T>::MergingDisallowed);

			Self::validate_signed(&transaction, Some(who))?;
			Self::update_storage(&transaction);
			Self::deposit_event(RawEvent::TransferSuccess(transaction))
		}

		/// Create a UTXO out of thin air for account `to` with value `value`.
		///
		/// Requires manager origin.
		#[weight = (0, Pays::No)]
		pub fn mint(origin, value: Value, to: T::AccountId) {

			let utxo = TransactionOutput { value, pubkey: to.clone() };
			let nonce = frame_system::Module::<T>::account_nonce(&to);
			let hash = BlakeTwo256::hash_of(&(&utxo, nonce));
			frame_system::Module::<T>::inc_account_nonce(&to);
			UtxoStore::<T>::insert(&hash, &utxo);
			OwnedUtxos::<T>::append(to, hash);

			Self::deposit_event(RawEvent::CreatedUtxo(hash, utxo));
		}
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Trait>::AccountId {
		/// Signed transaction was executed successfully
		TransferSuccess(Vec<Transaction<AccountId>>),
		/// A new UTXO was created at the given hash
		CreatedUtxo(UtxoHash, TransactionOutput<AccountId>),
	}
);

// "Internal" functions, callable by code.
impl<T: Trait> Module<T> {
	/// Check transaction for validity & errors
	///
	/// Ensures that:
	/// - inputs and outputs are not empty and below the size limit
	/// - all inputs match to existing, unspent and unlocked outputs
	/// - each input is used exactly once
	/// - each output has nonzero value
	/// - total output value must not exceed total input value
	/// - new outputs do not collide with existing ones
	/// - sum of input and output values does not overflow
	/// - the UTXOs belong to the given account (if account is passed)
	/// - transaction outputs cannot be modified by malicious nodes
	fn validate_signed(tx: &[Transaction<T::AccountId>], account: Option<T::AccountId>) -> DispatchResult {
		ensure!(!tx.is_empty(), Error::<T>::EmptyTransaction);
		ensure!(tx.len() <= MAX_INPUTS.min(MAX_OUTPUTS), Error::<T>::TransactionTooBig);

		for pair in tx.iter() {
			ensure!(!pair.input.is_empty(), Error::<T>::EmptyInput);
			ensure!(pair.input.len() <= MAX_INPUTS, Error::<T>::TooManyInputs);
			ensure!(!pair.output.is_empty(), Error::<T>::EmptyOutput);
			ensure!(pair.output.len() <= MAX_OUTPUTS, Error::<T>::TooManyOutputs);
		}

		{
			// check input uniqueness
			let input_set: BTreeMap<_, ()> = tx.iter()
				.map(|pair| &pair.input).flatten()
				.map(|o| (o, ())).collect();
			let input_count: usize = tx.iter().map(|pair| pair.input.len()).sum();
			ensure!(input_set.len() == input_count, Error::<T>::DuplicateInput);
		}

		let encoded_tx = tx.encode();

		let skip_ownership = account.is_none();
		for (pair_idx, pair) in tx.iter().enumerate() {
			let mut pair_input: Value = 0;
			// Check that inputs are valid
			for hash in pair.input.iter() {
				if let Some(input_utxo) = UtxoStore::<T>::get(&hash) {
					ensure!(skip_ownership || Some(input_utxo.pubkey) == account, Error::<T>::NotOwner);
					pair_input = pair_input.checked_add(input_utxo.value).ok_or(Error::<T>::Overflow)?;
				} else {
					return Err(Error::<T>::InputUtxoMissing.into());
				}
			}

			let mut pair_output: Value = 0;
			// Check that outputs are valid
			// TODO consider checking output hash uniqueness
			for (output_idx, output) in pair.output.iter().enumerate() {
				ensure!(output.value > 0, Error::<T>::OutputIsZero);
				let hash = Self::output_hash(&encoded_tx, pair_idx as u64, output_idx as u64);
				ensure!(!UtxoStore::<T>::contains_key(hash), Error::<T>::OutputAlreadyExists);
				pair_output = pair_output.checked_add(output.value).ok_or(Error::<T>::Overflow)?;
			}
			ensure!(pair_input >= pair_output, Error::<T>::OutputGreaterThanInput);
		}

		Ok(())
	}

	pub fn output_hash(encoded_tx: &[u8], pair_idx: u64, output_idx: u64) -> TxHash {
		BlakeTwo256::hash_of(&(encoded_tx, pair_idx, output_idx))
	}

	/// Update storage to reflect changes made by transaction
	/// Where each utxo key is a hash of the entire transaction and its order in the TransactionOutputs vector
	fn update_storage(tx: &[Transaction<T::AccountId>]) {
		use frame_support::debug;
		debug::RuntimeLogger::init();

		let encoded_tx = tx.encode();
		for (pair_idx, pair) in tx.iter().enumerate() {

			// Remove spent UTXOs
			for (_, hash) in pair.input.iter().enumerate() {
				if let Some(input_utxo) = UtxoStore::<T>::take(hash) {
					Self::remove_ownership(hash, &input_utxo.pubkey);
				} else {
					// Should not happen if transaction was validated properly.
					debug::error!("Utxo missing with hash '{:?}'", hash);
				}
			}

			// Store new UTXOs
			for (output_idx, output) in pair.output.iter().enumerate() {
				let hash = Self::output_hash(&encoded_tx, pair_idx as u64, output_idx as u64);
				UtxoStore::<T>::insert(&hash, output);
				OwnedUtxos::<T>::append(&output.pubkey, hash);
				Self::deposit_event(RawEvent::CreatedUtxo(hash.clone(), output.clone()));
			}
		}
	}

	/// Remove ownership of the UTXO at `hash` for `account`.
	pub fn remove_ownership(hash: &UtxoHash, account: &T::AccountId) {
		OwnedUtxos::<T>::mutate(account, |maybe_hashes| {
			let mut hashes = maybe_hashes.take().unwrap_or(Vec::new());
			hashes.retain(|h| h != hash);
			*maybe_hashes = if !hashes.is_empty() {
				Some(hashes)
			} else {
				None
			}
		});
	}
}
