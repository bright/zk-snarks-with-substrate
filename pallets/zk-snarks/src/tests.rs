// MIT License

// Copyright (c) 2022 Bright Inventions

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![cfg(test)]

use crate::{mock::*, *};

use frame_support::{assert_err, assert_ok};

#[test]
fn test_setup_verification() {
	new_test_ext().execute_with(|| {
		assert_ok!(ZKSnarks::setup_verification(RuntimeOrigin::none(), 50, br#"1234567"#.to_vec()));
		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}

#[test]
fn test_to_long_verification_key() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				50,
				vec![0; (<Test as Config>::MaxVerificationKeyLength::get() + 1) as usize]
			),
			Error::<Test>::TooLongVerificationKey
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_to_long_proof() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(
				RuntimeOrigin::none(),
				vec![0; (<Test as Config>::MaxProofLength::get() + 1) as usize]
			),
			Error::<Test>::TooLongProof
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_to_short_proof() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(
				RuntimeOrigin::none(),
				Vec::new()
			),
			Error::<Test>::ProofIsEmpty
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verify_without_verification_key() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), br#"1234567"#.to_vec()),
			Error::<Test>::VerificationKeyIsNotSet
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verification_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(ZKSnarks::setup_verification(RuntimeOrigin::none(), 7, br#"1234567"#.to_vec()));
		assert_ok!(ZKSnarks::verify(RuntimeOrigin::none(), vec![0; 50]));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(events[2], Event::<Test>::VerificationFailed);
	});
}

#[test]
fn test_verification_success() {
	new_test_ext().execute_with(|| {
		assert_ok!(ZKSnarks::setup_verification(RuntimeOrigin::none(), 7, br#"1234567"#.to_vec()));
		assert_ok!(ZKSnarks::verify(RuntimeOrigin::none(), vec![0; 7]));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(events[2], Event::<Test>::VerificationSuccess);
	});
}
