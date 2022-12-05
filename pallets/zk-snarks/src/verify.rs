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

pub fn verify(vk: VerificationKey, proof: Proof, inputs: PublicInputs) -> VerificationResult {
	let public_inputs: &[<Bls12 as Engine>::Fr] = &inputs;

	if (public_inputs.len() + 1) != vk.ic.len() {
		return Err(InvalidVerificationKey)
	}

	let mut acc = vk.ic[0].to_curve();
	for (i, b) in public_inputs.iter().zip(vk.ic.iter().skip(1)) {
		AddAssign::<&<Bls12 as Engine>::G1>::add_assign(&mut acc, &(*b * i));
	}

	//lhs
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
	use bls12_381::{G1Affine, G2Affine, Scalar};
	use std::ops::Deref;

	const ALPHA_X: &str = "2635983656263320256511463995836413167331869092392943593306076905516259749312747842295447349507189592731785901862558";

	const BETA_X_C0: &str = "1296094501238138520689116246487755613076576267512760150298482409401507546730337296489670510433482825207790998397346";
	const BETA_X_C1: &str = "3467549840163329914429787393326495235851806074050417925094845001935796859739058829480949031354270816778136382040361";

	const GAMMA_X_C1: &str = "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758";
	const GAMMA_X_C0: &str = "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160";

	const DELTA_X_C1: &str = "3218306808275776807419693666072599084905639169987324420818366627509865827220976650759812930713956208246000627242485";
	const DELTA_X_C0: &str = "3284950120787447527021651154232749836311526699432807747366843303661566424019671125328694916562846460813145647040459";

	const PI_A_X: &str = "2820173869801000183955769496344276101575675010174203082588560105436284422640780128231242184109173767085197647834267";

	const PI_B_X_C1: &str = "3343285764332210309442703216841128605678475246673133285314301861643378387001264758819444434632415207857557469906035";
	const PI_B_X_C0: &str = "54413090665354594353317256815335793052197307111690011609872716599840507808706991989403605000342095250665180513594";

	const PI_C_X: &str = "3006923877346016048391409264528383002939756547318806158402407618139299715778986391175418348881376388499383266389442";

	const IC_1_X: &str = "3759794041598018594287463133849401670165044879836734797942436987012929463856866218164906521458646350224910548839839";
	const IC_2_X: &str = "3305881491744710205856868316456114914540772066725994230747514104922282269209779243587827394909802115252764372519712";

	construct_uint! {
		pub struct U256(6);
	}

	#[test]
	fn verify_correct_proof() {
		//----------VK------------//
		let alpha: G1Affine = create_g1(ALPHA_X).unwrap();

		let beta: G2Affine = create_g2(BETA_X_C1, BETA_X_C0).unwrap();

		let gamma: G2Affine = create_g2(GAMMA_X_C1, GAMMA_X_C0).unwrap();
		let delta: G2Affine = create_g2(DELTA_X_C1, DELTA_X_C0).unwrap();
		//----------END OF VK------------//

		//----------PROOF---------------//
		let pi_a: G1Affine = create_g1(PI_A_X).unwrap();
		let pi_b: G2Affine = create_g2(PI_B_X_C1, PI_B_X_C0).unwrap();
		let pi_c: G1Affine = create_g1(PI_C_X).unwrap();
		let ic_1: G1Affine = create_g1(IC_1_X).unwrap();
		let ic_2: G1Affine = create_g1(IC_2_X).unwrap();

		//----------VERIFICATION---------------//
		let ic = vec![ic_1, ic_2];
		let public_inputs: [Scalar; 1] = [33.into()];
		let vk = VerificationKey { alpha, beta, gamma, delta, ic };
		let proof = Proof { a: pi_a, b: pi_b, c: pi_c };
		let result = verify(Some(vk), Some(proof), Some(public_inputs.into())).unwrap();
		assert!(result)
	}

	#[test]
	fn verify_incorrect_proof() {
		//----------VK------------//
		let alpha: G1Affine = create_g1(ALPHA_X).unwrap();

		let beta: G2Affine = create_g2(BETA_X_C1, BETA_X_C0).unwrap();

		let gamma: G2Affine = create_g2(GAMMA_X_C1, GAMMA_X_C0).unwrap();
		let delta: G2Affine = create_g2(DELTA_X_C1, DELTA_X_C0).unwrap();
		//----------END OF VK------------//

		//----------PROOF---------------//
		let pi_a: G1Affine = create_g1(PI_C_X).unwrap();
		let pi_b: G2Affine = create_g2(PI_B_X_C1, PI_B_X_C0).unwrap();
		let pi_c: G1Affine = create_g1(PI_A_X).unwrap();
		let ic_1: G1Affine = create_g1(IC_1_X).unwrap();
		let ic_2: G1Affine = create_g1(IC_2_X).unwrap();

		//----------VERIFICATION---------------//
		let ic = vec![ic_1, ic_2];
		let public_inputs: [Scalar; 1] = [33.into()];
		let vk = VerificationKey { alpha, beta, gamma, delta, ic };
		let proof = Proof { a: pi_a, b: pi_b, c: pi_c };
		let result = verify(Some(vk), Some(proof), Some(public_inputs.into())).unwrap();
		assert!(!result)
	}

	fn create_g1(x: &str) -> Option<G1Affine> {
		let mut a: [u8; 48] = [0; 48];
		U256::from_dec_str(x).unwrap().to_big_endian(a.as_mut_slice());
		// https://github.com/zkcrypto/bls12_381/blob/main/src/g1.rs#L221
		a[0] |= 1u8 << 7;
		a[0] |= &0u8;
		a[0] |= &(1u8 << 5);
		G1Affine::from_compressed(&a).into()
	}

	fn create_g2(x_c1: &str, x_c0: &str) -> Option<G2Affine> {
		let mut beta_bytes: Vec<u8> = vec![];
		beta_bytes.extend_from_slice(&from_dec_string(x_c1));
		beta_bytes.extend_from_slice(&from_dec_string(x_c0));
		// https://github.com/zkcrypto/bls12_381/blob/main/src/g2.rs#L264
		beta_bytes[0] |= 1u8 << 7;
		beta_bytes[0] |= &0u8;
		beta_bytes[0] |= &(1u8 << 5);
		let beta_byts_arr: Box<[u8; 96]> = beta_bytes.into_boxed_slice().try_into().unwrap();
		G2Affine::from_compressed(beta_byts_arr.deref()).into()
	}

	fn from_dec_string(number: &str) -> [u8; 48] {
		let mut a: [u8; 48] = [0; 48];
		let alpha_1_x = U256::from_dec_str(number).unwrap();
		alpha_1_x.to_big_endian(a.as_mut_slice());
		a
	}
}
