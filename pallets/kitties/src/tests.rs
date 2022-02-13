#![cfg(test)]

use crate::{mock::*, pallet::Error, *};
use frame_support::{assert_noop, assert_ok};

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
		let kitty1 = Kitties::<Test>::get(kid1).unwrap();
		assert_eq!(kitty1.owner, 1);

		let kid2 = kitties_owned_by_2[0];
		let kitty2 = Kitties::<Test>::get(kid2).unwrap();
		assert_eq!(kitty2.owner, 2);
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
		assert_eq!(KittiesOwned::<Test>::get(10).len(), 1);

		// check that some random account #5 does not own a kitty
		assert_eq!(KittiesOwned::<Test>::get(5).len(), 0);

		// check that this kitty is specifically owned by account #10
		let hash = KittiesOwned::<Test>::get(10)[0];
		let kitty = Kitties::<Test>::get(hash).unwrap();
		assert_eq!(kitty.owner, 10);
		assert_eq!(kitty.price, None);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// check that account 10 own a kitty
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
		assert_eq!(KittiesOwned::<Test>::get(10).len(), 1);
		let hash = KittiesOwned::<Test>::get(10)[0];

		// account 10 send kitty to account 3
		assert_ok!(SubstrateKitties::transfer(Origin::signed(10), 3, hash));

		// account 10 now has nothing
		assert_eq!(KittiesOwned::<Test>::get(10).len(), 0);
		// but account 3 does
		assert_eq!(KittiesOwned::<Test>::get(3).len(), 1);
		let new_hash = KittiesOwned::<Test>::get(3)[0];
		// and it has the same hash
		assert_eq!(hash, new_hash);
	});
}

#[test]
fn transfer_non_owned_kitty_should_fail() {
	new_test_ext().execute_with(|| {
		let hash = KittiesOwned::<Test>::get(1)[0];

		// account 0 cannot transfer a kitty with this hash.
		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(9), 2, hash),
			Error::<Test>::NotKittyOwner
		);
	});
}
