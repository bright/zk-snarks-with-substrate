#![cfg(test)]

use crate::{mock::*, pallet::Error, *};
use frame_support::{assert_noop, assert_ok};

// This function checks that kitty ownership is set correctly in storage.
// This will panic if things are not correct.
fn assert_ownership(owner: u64, kitty_id: [u8; 16]) {
	// For a kitty to be owned it should exist.
	let kitty = Kitties::<Test>::get(kitty_id).unwrap();
	// The kitty's owner is set correctly.
	assert_eq!(kitty.owner, owner);

	for (check_owner, owned) in KittiesOwned::<Test>::iter() {
		if owner == check_owner {
			// Owner should have this kitty.
			assert!(owned.contains(&kitty_id));
		} else {
			// Everyone else should not.
			assert!(!owned.contains(&kitty_id));
		}
	}
}

#[test]
fn should_build_genesis_kitties() {
	new_test_ext().execute_with(|| {
		// Check we have 2 kitties, as specified
		assert_eq!(CountForKitties::<Test>::get(), 2);

		// Check owners own the correct amount of kitties
		let kitties_owned_by_1 = KittiesOwned::<Test>::get(1);
		assert_eq!(kitties_owned_by_1.len(), 1);

		let kitties_owned_by_2 = KittiesOwned::<Test>::get(2);
		assert_eq!(kitties_owned_by_2.len(), 1);

		// Check that kitties are owned correctly
		let kid1 = kitties_owned_by_1[0];
		assert_ownership(1, kid1);

		let kid2 = kitties_owned_by_2[0];
		assert_ownership(2, kid2);
	});
}

#[test]
fn create_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// create a kitty with account #10.
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));

		// check that 3 kitties exists (together with the two from genesis)
		assert_eq!(CountForKitties::<Test>::get(), 3);

		// check that account #10 owns 1 kitty
		let kitties_owned = KittiesOwned::<Test>::get(10);
		assert_eq!(kitties_owned.len(), 1);
		let id = kitties_owned.last().unwrap();
		assert_ownership(10, *id);

		// check that this kitty is specifically owned by account #10
		let kitty = Kitties::<Test>::get(id).unwrap();
		assert_eq!(kitty.owner, 10);
		assert_eq!(kitty.price, None);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// check that account 10 own a kitty
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
		let id = KittiesOwned::<Test>::get(10)[0];

		// account 10 send kitty to account 3
		assert_ok!(SubstrateKitties::transfer(Origin::signed(10), 3, id));

		// account 10 now has nothing
		assert_eq!(KittiesOwned::<Test>::get(10).len(), 0);
		// but account 3 does
		assert_eq!(KittiesOwned::<Test>::get(3).len(), 1);
		assert_ownership(3, id);
	});
}

#[test]
fn transfer_non_owned_kitty_should_fail() {
	new_test_ext().execute_with(|| {
		let hash = KittiesOwned::<Test>::get(1)[0];

		// account 0 cannot transfer a kitty with this hash.
		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(9), 2, hash),
			Error::<Test>::NotOwner
		);
	});
}

// TODO: Check mint fails when kitty id already exists.

// TODO: Check that create_kitty fails when user owns too many kitties.

// TODO: Check that multiple create_kitty calls work in a single block.

// TODO: Check that breed kitty works as expected.

// TODO: Check breed kitty checks the same owner.

// TODO: Check that breed kitty checks opposite gender.

// TODO: Test gen_dna and other dna functions behave as expected

// TODO: Check transfer fails when transferring to self

// TODO: Check transfer fails when no kitty exists

// TODO: Check transfer fails when recipient has too many kitties.

// TODO: Check buy_kitty works as expected.

// TODO: Check buy_kitty fails when bid price is too low

// TODO: Check buy_kitty transfers the right amount when bid price is too high

// TODO: Check buy_kitty fails when balance is too low

// TODO: Check buy_kitty fails when kitty is not for sale

// TODO: Check set_price works as expected

// TODO: Check set_price fails when not owner
