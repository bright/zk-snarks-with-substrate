use crate::verify::VerificationError::InvalidVerificationKey;
use bls12_381::{Bls12, G1Affine, G2Affine, Scalar};
use group::{prime::PrimeCurveAffine, Curve};
use pairing::{Engine, MultiMillerLoop};
use sp_std::{ops::AddAssign, prelude::*};

pub struct VerificationKey {
	pub alpha: G1Affine,
	pub beta: G2Affine,
	pub gamma: G2Affine,
	pub delta: G2Affine,
	pub ic: Vec<G1Affine>,
}

pub struct Proof {
	pub a: G1Affine,
	pub b: G2Affine,
	pub c: G1Affine,
}

#[derive(Debug)]
pub enum VerificationError {
	InvalidVerificationKey,
}

pub type VerificationResult = Result<bool, VerificationError>;

pub type PublicInputs = Vec<Scalar>;

pub fn verify(
	vk: Option<VerificationKey>,
	proof: Option<Proof>,
	inputs: Option<PublicInputs>,
) -> VerificationResult {
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

	//lhs
	//todo: read a_b_pairing from verification_key
	let a_b_pairing = Bls12::pairing(&proof.a, &proof.b);

	//rhs
	let final_result = Bls12::multi_miller_loop(&[
		(&vk.alpha, &vk.beta.into()),
		(&acc.to_affine(), &vk.gamma.into()),
		(&proof.c, &vk.delta.into()),
	])
	.final_exponentiation();

	if a_b_pairing == final_result {
		Ok(true)
	} else {
		Ok(false)
	}
}

#[cfg(test)]
mod tests {
	use crate::verify::{verify, Proof, VerificationKey};
	use bls12_381::{Bls12, G1Affine, G2Affine, G2Prepared, Scalar};
	use pairing::MultiMillerLoop;
	use std::ops::Deref;
	use subtle::{Choice, ConditionallySelectable};

	construct_uint! {
		pub struct U256(6);
	}

	#[test]
	fn verify_correct_proof() {
		//----------VK------------//
		let alpha: G1Affine = g1_from_dec_string("2635983656263320256511463995836413167331869092392943593306076905516259749312747842295447349507189592731785901862558").unwrap();
		let beta: G2Affine = g2_from_dec_string("3467549840163329914429787393326495235851806074050417925094845001935796859739058829480949031354270816778136382040361", "1296094501238138520689116246487755613076576267512760150298482409401507546730337296489670510433482825207790998397346").unwrap();
		let gamma: G2Affine = g2_from_dec_string("3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758", "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160").unwrap();
		let delta: G2Affine = g2_from_dec_string("3218306808275776807419693666072599084905639169987324420818366627509865827220976650759812930713956208246000627242485", "3284950120787447527021651154232749836311526699432807747366843303661566424019671125328694916562846460813145647040459").unwrap();
		let terms = &[(&alpha, &G2Prepared::from(beta))];
		//----------END OF VK------------//

		//----------PROOF---------------//
		let pi_a: G1Affine = g1_from_dec_string("2820173869801000183955769496344276101575675010174203082588560105436284422640780128231242184109173767085197647834267").unwrap();
		let pi_b: G2Affine = g2_from_dec_string("3343285764332210309442703216841128605678475246673133285314301861643378387001264758819444434632415207857557469906035", "54413090665354594353317256815335793052197307111690011609872716599840507808706991989403605000342095250665180513594").unwrap();
		let pi_c: G1Affine = g1_from_dec_string("3006923877346016048391409264528383002939756547318806158402407618139299715778986391175418348881376388499383266389442").unwrap();
		let ic_1: G1Affine = g1_from_dec_string("3759794041598018594287463133849401670165044879836734797942436987012929463856866218164906521458646350224910548839839").unwrap();
		let ic_2: G1Affine = g1_from_dec_string("3305881491744710205856868316456114914540772066725994230747514104922282269209779243587827394909802115252764372519712").unwrap();

		//----------VERIFICATION---------------//
		let ic = vec![ic_1, ic_2];
		let public_inputs: [Scalar; 1] = [33.into()];
		let vk = VerificationKey { alpha, beta, gamma, delta, ic };
		let proof = Proof { a: pi_a, b: pi_b, c: pi_c };
		let result = verify(Some(vk), Some(proof), Some(public_inputs.into())).unwrap();
		assert!(result)
	}

	fn g1_from_dec_string(number: &str) -> Option<G1Affine> {
		let infinity: Choice = 0.into();
		let mut a: [u8; 48] = [0; 48];
		let alpha_1_x = U256::from_dec_str(number).unwrap();
		alpha_1_x.to_big_endian(a.as_mut_slice());
		//todo: not sure about these flags ...
		let y_lexicographically_largest: Choice = 0.into();
		//https://github.com/zkcrypto/bls12_381/blob/main/src/g1.rs#L221
		a[0] |= 1u8 << 7;
		a[0] |= u8::conditional_select(&0u8, &(1u8 << 6), infinity);
		a[0] |=
			u8::conditional_select(&0u8, &(1u8 << 5), (!infinity) & y_lexicographically_largest);
		G1Affine::from_compressed(&a).into()
	}

	fn g2_from_dec_string(number_1: &str, number_2: &str) -> Option<G2Affine> {
		let beta_1_bytes = &from_dec_string(number_1);
		let beta_2_bytes = &from_dec_string(number_2);
		let mut beta_bytes: Vec<u8> = vec![];
		beta_bytes.extend_from_slice(beta_1_bytes);
		beta_bytes.extend_from_slice(beta_2_bytes);
		let mut beta_byts_arr: Box<[u8; 96]> = match beta_bytes.into_boxed_slice().try_into() {
			Ok(ba) => ba,
			Err(o) => panic!("Expected a Vec of length {} but it was {}", 4, o.len()),
		};
		//todo: not sure about these flags ...
		let infinity = 0.into();
		let y_lexicographically_largest: Choice = 0.into();
		//https://github.com/zkcrypto/bls12_381/blob/main/src/g1.rs#L221
		(*beta_byts_arr)[0] |= 1u8 << 7;
		(*beta_byts_arr)[0] |= u8::conditional_select(&0u8, &(1u8 << 6), infinity);
		(*beta_byts_arr)[0] |=
			u8::conditional_select(&0u8, &(1u8 << 5), (!infinity) & y_lexicographically_largest);
		G2Affine::from_compressed(beta_byts_arr.deref()).into()
	}

	fn from_dec_string(number: &str) -> [u8; 48] {
		let mut a: [u8; 48] = [0; 48];
		let alpha_1_x = U256::from_dec_str(number).unwrap();
		alpha_1_x.to_big_endian(a.as_mut_slice());
		a
	}
}
