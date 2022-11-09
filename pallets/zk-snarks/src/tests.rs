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
