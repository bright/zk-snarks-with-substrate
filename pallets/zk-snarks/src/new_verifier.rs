use sp_std::ops::{AddAssign, Neg};
use bls12_381::{Bls12, G1Affine, G2Affine, G2Prepared, Scalar};
use group::Curve;
use group::prime::PrimeCurveAffine;
use pairing::{Engine, MultiMillerLoop};
use crate::new_verifier::VerificationError::InvalidVerificationKey;
use sp_std::prelude::*;


pub struct VerificationKey {
	pub alpha: G1Affine,
	pub beta: G2Affine,
	pub gamma: G2Affine,
	pub delta: G2Affine,
	pub ic: Vec<G1Affine>
}

pub struct Proof {
	pub a: G1Affine,
	pub b: G2Affine,
	pub c: G1Affine
}

#[derive(Debug)]
pub enum VerificationError {
	InvalidVerificationKey
}

pub type VerificationResult = Result<bool, VerificationError>;

pub type PublicInputs = Vec<Scalar>;

pub fn verify(vk: Option<VerificationKey>, proof: Option<Proof>, inputs: Option<PublicInputs>) -> VerificationResult {

	if vk.is_none() || proof.is_none() || inputs.is_none() {
		return Ok(true)
	}

	let vk = vk.unwrap();
	let proof = proof.unwrap();
	let inputs = inputs.unwrap();
	let public_inputs: &[<Bls12 as Engine>::Fr] = &inputs;

	if (public_inputs.len() + 1) != vk.ic.len() {
		return Err(InvalidVerificationKey)
	}

	let mut acc = vk.ic[0].to_curve();
	for (i, b) in public_inputs.iter().zip(vk.ic.iter().skip(1)) {
		AddAssign::<&<Bls12 as Engine>::G1>::add_assign(&mut acc, &(*b * i));
	}

	let final_result = Bls12::multi_miller_loop(&[
		(&proof.a, &proof.b.into()),
		(&acc.to_affine(), &vk.gamma.neg().into()),
		(&proof.c, &vk.delta.neg().into()),
	])
		.final_exponentiation();

	let terms = &[(&vk.alpha, &G2Prepared::from(vk.beta))];
	let alpha_beta = Bls12::multi_miller_loop(terms);

	if alpha_beta.final_exponentiation()
		== final_result
	{
		Ok(true)
	} else {
		Ok(false)
	}

}
