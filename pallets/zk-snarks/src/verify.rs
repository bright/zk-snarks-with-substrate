use crate::verify::VerificationError::InvalidVerificationKey;
use bls12_381::{Bls12, G1Affine, G2Affine, Scalar};
use group::{prime::PrimeCurveAffine, Curve};
use pairing::{Engine, MultiMillerLoop};
use sp_std::{ops::AddAssign, prelude::*};

pub const SUPPORTED_CURVE: &str = "bls12381";
pub const SUPPORTED_PROTOCOL: &str = "groth16";

pub struct G1Bytes {
	inner: [u8; 48],
}

pub struct G2Bytes {
	inner: [u8; 96],
}

impl G1Bytes {
	pub fn new(x_bytes: &[u8; 48]) -> Self {
		let mut new_bytes = x_bytes.to_owned();
		// https://github.com/zkcrypto/bls12_381/blob/main/src/g1.rs#L221
		new_bytes[0] |= 1u8 << 7;
		new_bytes[0] |= &0u8;
		new_bytes[0] |= &(1u8 << 5);
		G1Bytes { inner: new_bytes }
	}
}

impl G2Bytes {
	pub fn new(x_c1_bytes: &[u8; 48], x_c0_bytes: &[u8; 48]) -> Self {
		let mut new_bytes: [u8; 96] = [0; 96];
		for i in 0..48 {
			new_bytes[i] = x_c1_bytes[i];
			new_bytes[48 + i] = x_c0_bytes[i];
		}
		// https://github.com/zkcrypto/bls12_381/blob/main/src/g2.rs#L264
		new_bytes[0] |= 1u8 << 7;
		new_bytes[0] |= &0u8;
		new_bytes[0] |= &(1u8 << 5);
		G2Bytes { inner: new_bytes }
	}
}

impl TryFrom<&G1Bytes> for G1Affine {
	type Error = ();

	fn try_from(value: &G1Bytes) -> Result<Self, Self::Error> {
		let g1 = G1Affine::from_compressed(&value.inner);
		//todo: doesnt look nice
		if g1.is_none().into() {
			Err(())
		} else {
			Ok(g1.unwrap())
		}
	}
}

impl TryFrom<&G2Bytes> for G2Affine {
	type Error = ();

	fn try_from(value: &G2Bytes) -> Result<Self, Self::Error> {
		let g2 = G2Affine::from_compressed(&value.inner);
		if g2.is_none().into() {
			Err(())
		} else {
			Ok(g2.unwrap())
		}
	}
}

pub struct VerificationKey {
	pub alpha: G1Affine,
	pub beta: G2Affine,
	pub gamma: G2Affine,
	pub delta: G2Affine,
	pub ic: Vec<G1Affine>,
}

impl VerificationKey {
	pub fn from(
		alpha: &G1Bytes,
		beta: &G2Bytes,
		gamma: &G2Bytes,
		delta: &G2Bytes,
		ic: &Vec<G1Bytes>,
	) -> Result<Self, ()> {
		let alpha = alpha.try_into()?;
		let beta: G2Affine = beta.try_into()?;
		let gamma: G2Affine = gamma.try_into()?;
		let delta: G2Affine = delta.try_into()?;
		let mut ic_2: Vec<G1Affine> = Vec::with_capacity(ic.len());

		for i in 0..ic.len() {
			ic_2.push(G1Affine::try_from(&ic[i])?);
		}

		Ok(VerificationKey { alpha, beta, gamma, delta, ic: ic_2 })
	}
}

pub struct Proof {
	pub a: G1Affine,
	pub b: G2Affine,
	pub c: G1Affine,
}

impl Proof {
	pub fn from(a: &G1Bytes, b: &G2Bytes, c: &G1Bytes) -> Result<Self, ()> {
		let a = a.try_into()?;
		let b = b.try_into()?;
		let c = c.try_into()?;

		Ok(Proof { a, b, c })
	}
}

#[derive(Debug, PartialEq)]
pub enum VerificationError {
	InvalidVerificationKey,
}

pub type VerificationResult = Result<bool, VerificationError>;

pub type PublicInputs = Vec<Scalar>;

pub fn prepare_public_inputs(inputs: Vec<u64>) -> Vec<Scalar> {
	inputs.into_iter().map(|i| Scalar::from(i)).collect()
}

pub fn verify(vk: VerificationKey, proof: Proof, inputs: PublicInputs) -> VerificationResult {
	let public_inputs: &[<Bls12 as Engine>::Fr] = &inputs;

	if (public_inputs.len() + 1) != vk.ic.len() {
		return Err(InvalidVerificationKey)
	}

	// ic contains Lᵢ(τ)/δ
	// Lᵢ(x) = β * Aᵢ(x) + α * Bᵢ(x) + Cᵢ(x)
	// public variables [33]
	// w = [1, 33, ...private variables]
	// acc contains sum of Lᵢ(x) * wᵢ
	let mut acc = vk.ic[0].to_curve();
	for (i, b) in public_inputs.iter().zip(vk.ic.iter().skip(1)) {
		AddAssign::<&<Bls12 as Engine>::G1>::add_assign(&mut acc, &(*b * i));
	}

	//lhs
	// Aₚ*Bₚ
	let a_b_pairing = Bls12::pairing(&proof.a, &proof.b);

	//rhs
	// αβ + (L_input(τ)/γ)γ + Cₚδ
	let final_result = Bls12::multi_miller_loop(&[
		(&vk.alpha, &vk.beta.into()),
		(&acc.to_affine(), &vk.gamma.into()),
		(&proof.c, &vk.delta.into()),
	])
	.final_exponentiation();

	Ok(a_b_pairing == final_result)
}

#[cfg(test)]
mod tests {
	use crate::verify::{verify, G1Bytes, G2Bytes, Proof, VerificationError, VerificationKey};
	use bls12_381::{G1Affine, G2Affine};
	use frame_support::assert_ok;
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
	fn verification_key_from_correct_coordinates_is_ok() {
		let vk = VerificationKey::from(
			&G1Bytes::new(&from_dec_string(ALPHA_X)),
			&G2Bytes::new(&from_dec_string(BETA_X_C1), &from_dec_string(BETA_X_C0)),
			&G2Bytes::new(&from_dec_string(GAMMA_X_C1), &from_dec_string(GAMMA_X_C0)),
			&G2Bytes::new(&from_dec_string(DELTA_X_C1), &from_dec_string(DELTA_X_C0)),
			&vec![G1Bytes::new(&from_dec_string(IC_1_X)), G1Bytes::new(&from_dec_string(IC_2_X))],
		);

		assert!(vk.is_ok())
	}

	#[test]
	fn proof_from_correct_coordinates_is_ok() {
		let proof = Proof::from(
			&G1Bytes::new(&from_dec_string(PI_A_X)),
			&G2Bytes::new(&from_dec_string(PI_B_X_C1), &from_dec_string(PI_B_X_C0)),
			&G1Bytes::new(&from_dec_string(PI_C_X)),
		);
		assert!(proof.is_ok())
	}

	#[test]
	fn verify_correct_proof() {
		// circuit description https://github.com/iden3/circom/blob/7e59274c3e78674c2178766f9b8a4371c760ac3a/mkdocs/docs/getting-started/writing-circuits.md

		//----------VK------------//
		// sample/verification_key.json
		let alpha: G1Affine = create_g1(ALPHA_X).unwrap();

		let beta: G2Affine = create_g2(BETA_X_C1, BETA_X_C0).unwrap();

		let gamma: G2Affine = create_g2(GAMMA_X_C1, GAMMA_X_C0).unwrap();
		let delta: G2Affine = create_g2(DELTA_X_C1, DELTA_X_C0).unwrap();
		//----------END OF VK------------//

		//----------PROOF---------------//
		// sample/proof.json
		let pi_a: G1Affine = create_g1(PI_A_X).unwrap();
		let pi_b: G2Affine = create_g2(PI_B_X_C1, PI_B_X_C0).unwrap();
		let pi_c: G1Affine = create_g1(PI_C_X).unwrap();
		let ic_1: G1Affine = create_g1(IC_1_X).unwrap();
		let ic_2: G1Affine = create_g1(IC_2_X).unwrap();
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert!(verify(
			VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1, ic_2] },
			Proof { a: pi_a, b: pi_b, c: pi_c },
			// sample/public.json
			[33.into()].into(),
		)
		.unwrap())
		//--------END OF VERIFICATION---------//
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
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert!(!verify(
			VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1, ic_2] },
			Proof { a: pi_a, b: pi_b, c: pi_c },
			[33.into()].into(),
		)
		.unwrap())
		//--------END OF VERIFICATION---------//
	}

	#[test]
	fn verify_with_incorrect_ic_len() {
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
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert_eq!(
			verify(
				VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1] },
				Proof { a: pi_a, b: pi_b, c: pi_c },
				[33.into()].into(),
			)
			.err()
			.unwrap(),
			VerificationError::InvalidVerificationKey
		)
		//--------END OF VERIFICATION---------//
	}

	fn create_g1(x: &str) -> Result<G1Affine, ()> {
		G1Affine::try_from(&G1Bytes::new(&from_dec_string(x)))
	}

	fn create_g2(x_c1: &str, x_c0: &str) -> Result<G2Affine, ()> {
		G2Affine::try_from(&G2Bytes::new(&from_dec_string(x_c1), &from_dec_string(x_c0)))
	}

	fn from_dec_string(number: &str) -> [u8; 48] {
		let mut bytes: [u8; 48] = [0; 48];
		U256::from_dec_str(number).unwrap().to_big_endian(bytes.as_mut_slice());
		bytes
	}
}
