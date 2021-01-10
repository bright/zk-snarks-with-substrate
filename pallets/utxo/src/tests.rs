// Copyright 2020-2021 Parity Technologies (UK) Ltd.
use super::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};
use sp_core::testing::{KeyStore, SR25519};
use sp_core::{sr25519::Public, traits::KeystoreExt};
use sp_std::collections::btree_map::BTreeMap;

// need to manually import this crate since its no include by default
use hex_literal::hex;
use maplit::btreemap;

const ALICE_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
// other random accounts generated with subkey
const BOB_PHRASE: &str = "toast youth kingdom soft caution hand burst cliff scissors wisdom test letter";
const EVE_PHRASE: &str = "puzzle thing true express thumb hidden spring series render earn chimney essay";
const KARL_PHRASE: &str =
	"monitor exhibit resource stumble subject nut valid furnace obscure misery satoshi assume";
const GENESIS_UTXO: [u8; 32] = hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999");

// This function basically just builds a genesis storage key/value store according to our desired mockup.
// We start each test by giving Alice 100 utxo to start with.
fn new_test_ext_and_keys() -> (sp_io::TestExternalities, BTreeMap<&'static str, Public>) {
	let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
	let alice_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();
	let bob_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(BOB_PHRASE)).unwrap();
	let eve_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(EVE_PHRASE)).unwrap();
	let karl_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(KARL_PHRASE)).unwrap();

	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let _ = GenesisConfig::<Test> {
		genesis_utxos: vec![TransactionOutput {
			value: 100,
			pubkey: H256::from(alice_pub_key),
		}],
	}
	.assimilate_storage(&mut storage);

	// Print the values to get GENESIS_UTXO
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.register_extension(KeystoreExt(keystore));
	(
		ext,
		btreemap! {
			"Alice" => alice_pub_key,
			"Bob" => bob_pub_key,
			"Eve" => eve_pub_key,
			"Karl" => karl_pub_key,
		},
	)
}

// ----------------------
// helper functions

fn build_utxo<P>(value: Value, pubkey: P) -> TransactionOutput<H256>
where
	P: Into<H256>,
{
	TransactionOutput {
		value,
		pubkey: pubkey.into(),
	}
}

fn build_tx(outpoint: UtxoHash, value: Value, to: H256) -> Vec<Transaction<H256>> {
	vec![Transaction {
		input: vec![outpoint],
		output: vec![TransactionOutput { value, pubkey: to }],
	}]
}

fn transfer_utxo<I, T>(input: I, from: Public, to: T) -> UtxoHash
where
	I: Into<UtxoHash>,
	T: Into<H256>,
{
	let hash = input.into();
	let utxo = UtxoStore::<Test>::get(hash).unwrap();
	let transaction = build_tx(hash, utxo.value, to.into());
	let receiver_utxo_hash = Utxo::output_hash(&transaction.encode(), 0, 0);
	assert_ok!(Utxo::send_transaction(Origin::signed(from.into()), transaction));
	receiver_utxo_hash
}

fn transfer_amount<I, T>(input: I, from: Public, amount: Value, to: T) -> (UtxoHash, UtxoHash)
where
	I: Into<UtxoHash>,
	T: Into<H256>,
{
	let hash = input.into();
	let utxo = UtxoStore::<Test>::get(hash).unwrap();
	assert!(utxo.value >= amount, "utxo value too small");
	let transaction = vec![Transaction {
		input: vec![hash],
		output: vec![
			TransactionOutput {
				value: utxo.value - amount,
				pubkey: from.into(),
			},
			TransactionOutput {
				value: amount,
				pubkey: to.into(),
			},
		],
	}];
	let encoded_tx = transaction.encode();
	let sender_utxo_hash = Utxo::output_hash(&encoded_tx, 0, 0);
	let receiver_utxo_hash = Utxo::output_hash(&encoded_tx, 0, 1);
	assert_ok!(Utxo::send_transaction(Origin::signed(from.into()), transaction));
	(sender_utxo_hash, receiver_utxo_hash)
}

fn split<I, T>(input: I, from: Public, output: Vec<T>) -> Vec<UtxoHash>
where
	I: Into<UtxoHash>,
	T: Into<TransactionOutput<H256>>,
{
	let hash = input.into();
	let utxo = UtxoStore::<Test>::get(hash).unwrap();
	let output: Vec<_> = output.into_iter().map(|o| o.into()).collect();
	assert!(
		utxo.value >= output.iter().map(|o| o.value).sum(),
		"utxo does not cover split value"
	);
	let transaction = vec![Transaction {
		input: vec![hash],
		output: output.clone(),
	}];
	let encoded_tx = transaction.encode();
	let hashes = output
		.iter()
		.enumerate()
		.map(|(idx, _o)| Utxo::output_hash(&encoded_tx, 0, idx as u64))
		.collect();
	assert_ok!(Utxo::send_transaction(Origin::signed(from.into()), transaction));
	hashes
}

// </helper functions>
// ----------------------

#[test]
fn simple_signed_transaction_works() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	test_ext.execute_with(|| {
		// Alice wants to send herself a new utxo of value 50.
		let transaction = vec![Transaction {
			input: vec![UtxoHash::from(GENESIS_UTXO)],
			output: vec![build_utxo(50, alice_pub_key)],
		}];

		let new_utxo_hash = Utxo::output_hash(&transaction.encode(), 0, 0);

		assert_ok!(Utxo::send_transaction(Origin::signed(alice_pub_key.into()), transaction));
		assert!(!UtxoStore::<Test>::contains_key(H256::from(GENESIS_UTXO)));
		assert!(UtxoStore::<Test>::contains_key(new_utxo_hash));
		assert_eq!(50, UtxoStore::<Test>::get(new_utxo_hash).unwrap().value);
	});
}

#[test]
fn transfer_works() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	let bob_pub_key = keys["Bob"];

	let genesis_utxo_hash = UtxoHash::from(GENESIS_UTXO);

	test_ext.execute_with(|| {
		let transaction = vec![Transaction {
			input: vec![genesis_utxo_hash],
			output: vec![build_utxo(100, bob_pub_key)],
		}];

		let new_utxo_hash = Utxo::output_hash(&transaction.encode(), 0, 0);

		assert_ok!(Utxo::transfer(Origin::signed(alice_pub_key.into()), vec![genesis_utxo_hash], bob_pub_key.into()));
		assert!(!UtxoStore::<Test>::contains_key(genesis_utxo_hash));
		assert!(UtxoStore::<Test>::contains_key(new_utxo_hash));
		assert_eq!(100, UtxoStore::<Test>::get(new_utxo_hash).unwrap().value);
	});
}

#[test]
fn mint_works() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let bob_pub_key = keys["Bob"];

	test_ext.execute_with(|| {
		let value = 42;
		let bob_acc: H256 = bob_pub_key.into();
		let utxo = TransactionOutput { value, pubkey: bob_acc.clone() };
		let nonce = frame_system::Module::<Test>::account_nonce(&bob_acc);
		let new_utxo_hash = BlakeTwo256::hash_of(&(&utxo, nonce));

		assert_ok!(Utxo::mint(Origin::root(), value, bob_acc));
		assert!(UtxoStore::<Test>::contains_key(new_utxo_hash));
		assert_eq!(value, UtxoStore::<Test>::get(new_utxo_hash).unwrap().value);
	});
}

#[test]
fn attack_with_sending_to_own_account() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let karl_pub_key = keys["Karl"];
	test_ext.execute_with(|| {
		// Karl wants to send himself a new utxo of value 50 out of thin air.
		let transaction = build_tx(UtxoHash::zero(), 50, H256::from(karl_pub_key));

		assert_noop!(
			Utxo::send_transaction(Origin::signed(karl_pub_key.into()), transaction),
			Error::<Test>::InputUtxoMissing
		);
	});
}

#[test]
fn attack_with_empty_transactions() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	test_ext.execute_with(|| {
		assert_noop!(
			Utxo::send_transaction(Origin::signed(alice_pub_key.into()), Vec::new()), // an empty trx
			Error::<Test>::EmptyTransaction
		);

		assert_noop!(
			Utxo::send_transaction(
				Origin::signed(alice_pub_key.into()),
				vec![Transaction {
					input: vec![UtxoHash::default()], // an empty trx
					output: vec![],
				}]
			),
			Error::<Test>::EmptyOutput
		);
	});
}

#[test]
fn attack_by_permanently_sinking_outputs() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	test_ext.execute_with(|| {

		let transaction = vec![Transaction {
			input: vec![H256::from(GENESIS_UTXO)],
			// A 0 value output burns this output forever!
			output: vec![TransactionOutput {
				value: 0,
				pubkey: H256::from(alice_pub_key),
			}],
		}];

		assert_noop!(
			Utxo::send_transaction(Origin::signed(alice_pub_key.into()), transaction),
			Error::<Test>::OutputIsZero
		);
	});
}

#[test]
fn attack_by_overflowing_value() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	test_ext.execute_with(|| {

		let transaction = vec![Transaction {
			input: vec![H256::from(GENESIS_UTXO)],
			output: vec![
				TransactionOutput {
					value: Value::max_value(),
					pubkey: H256::from(alice_pub_key),
				},
				// Attempts to do overflow total output value
				TransactionOutput {
					value: 10 as Value,
					pubkey: H256::from(alice_pub_key),
				},
			],
		}];

		assert_noop!(
			Utxo::send_transaction(Origin::signed(alice_pub_key.into()), transaction),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn attack_by_over_spending() {
	let (mut test_ext, keys) = new_test_ext_and_keys();
	let alice_pub_key = keys["Alice"];
	test_ext.execute_with(|| {

		let transaction = vec![Transaction {
			input: vec![H256::from(GENESIS_UTXO)],
			output: vec![
				TransactionOutput {
					value: 100 as Value,
					pubkey: H256::from(alice_pub_key),
				},
				// Creates 2 new utxo out of thin air!
				TransactionOutput {
					value: 2 as Value,
					pubkey: H256::from(alice_pub_key),
				},
			],
		}];

		assert_noop!(
			Utxo::send_transaction(Origin::signed(alice_pub_key.into()), transaction),
			Error::<Test>::OutputGreaterThanInput
		);
	});
}
