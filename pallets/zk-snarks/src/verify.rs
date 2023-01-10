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


use crate::verify::VerificationError::InvalidVerificationKey;
use bls12_381::{Bls12, G1Affine, G2Affine, Scalar};
use group::{prime::PrimeCurveAffine, Curve};
use pairing::{Engine, MultiMillerLoop};
use sp_std::{ops::AddAssign, prelude::*};

pub const SUPPORTED_CURVE: &str = "bls12381";
pub const SUPPORTED_PROTOCOL: &str = "groth16";

pub struct G1UncompressedBytes {
	inner: [u8; 96],
}

pub struct G2UncompressedBytes {
	inner: [u8; 192],
}

impl G1UncompressedBytes {
	pub fn new(x: [u8; 48], y: [u8; 48]) -> Self {
		let mut new_bytes: [u8; 96] = [0; 96];

		for i in 0..48 {
			new_bytes[i] = x[i];
			new_bytes[i + 48] = y[i];
		}
		new_bytes[0] |= &0u8;

		G1UncompressedBytes { inner: new_bytes }
	}
}

impl G2UncompressedBytes {
	pub fn new(x_c0: [u8; 48], x_c1: [u8; 48], y_c0: [u8; 48], y_c1: [u8; 48]) -> Self {
		let mut new_bytes: [u8; 192] = [0; 192];

		for i in 0..48 {
			new_bytes[i] = x_c1[i];
			new_bytes[i + 48] = x_c0[i];
			new_bytes[i + 96] = y_c1[i];
			new_bytes[i + 144] = y_c0[i];
		}
		new_bytes[0] |= &0u8;

		G2UncompressedBytes { inner: new_bytes }
	}
}

impl TryFrom<&G1UncompressedBytes> for G1Affine {
	type Error = ();

	fn try_from(value: &G1UncompressedBytes) -> Result<Self, Self::Error> {
		let g1 = G1Affine::from_uncompressed(&value.inner);
		if g1.is_none().into() {
			Err(())
		} else {
			Ok(g1.unwrap())
		}
	}
}

impl TryFrom<&G2UncompressedBytes> for G2Affine {
	type Error = ();

	fn try_from(value: &G2UncompressedBytes) -> Result<Self, Self::Error> {
		let g2 = G2Affine::from_uncompressed(&value.inner);
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
	pub fn from_uncompressed(
		alpha: &G1UncompressedBytes,
		beta: &G2UncompressedBytes,
		gamma: &G2UncompressedBytes,
		delta: &G2UncompressedBytes,
		ic: &Vec<G1UncompressedBytes>,
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

pub struct GProof {
	pub a: G1Affine,
	pub b: G2Affine,
	pub c: G1Affine,
}

impl GProof {
	pub fn from_uncompressed(
		a: &G1UncompressedBytes,
		b: &G2UncompressedBytes,
		c: &G1UncompressedBytes,
	) -> Result<Self, ()> {
		let a = a.try_into()?;
		let b = b.try_into()?;
		let c = c.try_into()?;

		Ok(GProof { a, b, c })
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

pub fn verify(vk: VerificationKey, proof: GProof, inputs: PublicInputs) -> VerificationResult {
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
	use crate::verify::{
		verify, G1UncompressedBytes, G2UncompressedBytes, GProof, VerificationError, VerificationKey,
	};
	use bls12_381::{G1Affine, G2Affine};

	const ALPHA_X: &str = "2635983656263320256511463995836413167331869092392943593306076905516259749312747842295447349507189592731785901862558";
	const ALPHA_Y: &str = "743892996456702519498029594549937288641619275055957975879157306988929970626325326222697609972550552691064908651931";

	const BETA_X_C0: &str = "1296094501238138520689116246487755613076576267512760150298482409401507546730337296489670510433482825207790998397346";
	const BETA_X_C1: &str = "3467549840163329914429787393326495235851806074050417925094845001935796859739058829480949031354270816778136382040361";

	const BETA_Y_C0: &str = "3403410200913851046378881164751590587066009874691619938225021193334979700147466129997648606538377099567064346931273";
	const BETA_Y_C1: &str = "3804847074485539411700684267722735363688167108429634491643293100788171321105199556340902873511607444652008144844173";

	const GAMMA_X_C1: &str = "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758";
	const GAMMA_X_C0: &str = "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160";

	const GAMMA_Y_C1: &str = "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582";
	const GAMMA_Y_C0: &str = "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905";

	const DELTA_X_C1: &str = "3218306808275776807419693666072599084905639169987324420818366627509865827220976650759812930713956208246000627242485";
	const DELTA_X_C0: &str = "3284950120787447527021651154232749836311526699432807747366843303661566424019671125328694916562846460813145647040459";

	const DELTA_Y_C1: &str = "3505020872425170466568261366418107787485649574216477007429328593907934456720034754142706045279478244784882964969099";
	const DELTA_Y_C0: &str = "3945290144137392347873751586031392152201459997902585432454016489727689337013866944382877542451368688652560743518350";

	const PI_A_X: &str = "2820173869801000183955769496344276101575675010174203082588560105436284422640780128231242184109173767085197647834267";
	const PI_A_Y: &str = "1152541093585973172499551859168528642628429504007613830168996825879806250289422935864437193085184388469171892221011";

	const PI_B_X_C1: &str = "3343285764332210309442703216841128605678475246673133285314301861643378387001264758819444434632415207857557469906035";
	const PI_B_X_C0: &str = "54413090665354594353317256815335793052197307111690011609872716599840507808706991989403605000342095250665180513594";
	const PI_B_Y_C1: &str = "3777303780170739988308854254585940898119682705621212814969008224084499326863117961704608873229314936725151172212883";
	const PI_B_Y_C0: &str = "262180403851765105493218367619507205740764669171746348153545090105487398261554724750259283942935411519021270362742";

	const PI_C_X: &str = "3006923877346016048391409264528383002939756547318806158402407618139299715778986391175418348881376388499383266389442";
	const PI_C_Y: &str = "1307513151230758506579817970515482216448699470263630520204374492458260823057506418477833081567163581258564509876945";

	const IC_1_X: &str = "3759794041598018594287463133849401670165044879836734797942436987012929463856866218164906521458646350224910548839839";
	const IC_1_Y: &str = "3238512100593065266229132824040292706800754984648723917955334599968665051423411534393542324672325614522917210582797";

	const IC_2_X: &str = "3305881491744710205856868316456114914540772066725994230747514104922282269209779243587827394909802115252764372519712";
	const IC_2_Y: &str = "2462443929524735084767395208674598757462820081953985438437610428598624587728712969052746628125821805697605346885091";

	construct_uint! {
		pub struct U256(6);
	}

	#[test]
	fn verification_key_from_correct_coordinates_is_ok() {
		let vk = VerificationKey::from_uncompressed(
			&G1UncompressedBytes::new(from_dec_string(ALPHA_X), from_dec_string(ALPHA_Y)),
			&G2UncompressedBytes::new(
				from_dec_string(BETA_X_C0),
				from_dec_string(BETA_X_C1),
				from_dec_string(BETA_Y_C0),
				from_dec_string(BETA_Y_C1),
			),
			&G2UncompressedBytes::new(
				from_dec_string(GAMMA_X_C0),
				from_dec_string(GAMMA_X_C1),
				from_dec_string(GAMMA_Y_C0),
				from_dec_string(GAMMA_Y_C1),
			),
			&G2UncompressedBytes::new(
				from_dec_string(DELTA_X_C0),
				from_dec_string(DELTA_X_C1),
				from_dec_string(DELTA_Y_C0),
				from_dec_string(DELTA_Y_C1),
			),
			&vec![
				G1UncompressedBytes::new(from_dec_string(IC_1_X), from_dec_string(IC_1_Y)),
				G1UncompressedBytes::new(from_dec_string(IC_2_X), from_dec_string(IC_2_Y)),
			],
		);
		assert!(vk.is_ok())
	}

	#[test]
	fn proof_from_correct_coordinates_is_ok() {
		let proof = GProof::from_uncompressed(
			&G1UncompressedBytes::new(from_dec_string(PI_A_X), from_dec_string(PI_A_Y)),
			&G2UncompressedBytes::new(
				from_dec_string(PI_B_X_C0),
				from_dec_string(PI_B_X_C1),
				from_dec_string(PI_B_Y_C0),
				from_dec_string(PI_B_Y_C1),
			),
			&G1UncompressedBytes::new(from_dec_string(PI_C_X), from_dec_string(PI_C_Y)),
		);
		assert!(proof.is_ok())
	}

	#[test]
	fn verify_correct_proof() {
		// circuit description https://github.com/iden3/circom/blob/7e59274c3e78674c2178766f9b8a4371c760ac3a/mkdocs/docs/getting-started/writing-circuits.md

		//----------VK------------//
		// sample/verification_key.json
		let alpha: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(ALPHA_X), from_dec_string(ALPHA_Y)))
				.try_into()
				.unwrap();

		let beta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(BETA_X_C0),
			from_dec_string(BETA_X_C1),
			from_dec_string(BETA_Y_C0),
			from_dec_string(BETA_Y_C1),
		))
			.try_into()
			.unwrap();

		let gamma: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(GAMMA_X_C0),
			from_dec_string(GAMMA_X_C1),
			from_dec_string(GAMMA_Y_C0),
			from_dec_string(GAMMA_Y_C1),
		))
			.try_into()
			.unwrap();
		let delta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(DELTA_X_C0),
			from_dec_string(DELTA_X_C1),
			from_dec_string(DELTA_Y_C0),
			from_dec_string(DELTA_Y_C1),
		))
			.try_into()
			.unwrap();

		let ic_1: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(IC_1_X), from_dec_string(IC_1_Y)))
				.try_into()
				.unwrap();
		let ic_2: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(IC_2_X), from_dec_string(IC_2_Y)))
				.try_into()
				.unwrap();

		//----------END OF VK------------//

		//----------PROOF---------------//
		// sample/proof.json
		let pi_a: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_A_X), from_dec_string(PI_A_Y)))
				.try_into()
				.unwrap();
		let pi_b: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(PI_B_X_C0),
			from_dec_string(PI_B_X_C1),
			from_dec_string(PI_B_Y_C0),
			from_dec_string(PI_B_Y_C1),
		))
			.try_into()
			.unwrap();
		let pi_c: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_C_X), from_dec_string(PI_C_Y)))
				.try_into()
				.unwrap();
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert!(verify(
			VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1, ic_2] },
			GProof { a: pi_a, b: pi_b, c: pi_c },
			// sample/public.json
			[33.into()].into(),
		)
		.unwrap())
		//--------END OF VERIFICATION---------//
	}

	#[test]
	fn verify_incorrect_proof() {
		//----------VK------------//
		// sample/verification_key.json
		let alpha: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(ALPHA_X), from_dec_string(ALPHA_Y)))
				.try_into()
				.unwrap();

		let beta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(BETA_X_C0),
			from_dec_string(BETA_X_C1),
			from_dec_string(BETA_Y_C0),
			from_dec_string(BETA_Y_C1),
		))
			.try_into()
			.unwrap();

		let gamma: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(GAMMA_X_C0),
			from_dec_string(GAMMA_X_C1),
			from_dec_string(GAMMA_Y_C0),
			from_dec_string(GAMMA_Y_C1),
		))
			.try_into()
			.unwrap();
		let delta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(DELTA_X_C0),
			from_dec_string(DELTA_X_C1),
			from_dec_string(DELTA_Y_C0),
			from_dec_string(DELTA_Y_C1),
		))
			.try_into()
			.unwrap();

		let ic_1: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(IC_1_X), from_dec_string(IC_1_Y)))
				.try_into()
				.unwrap();
		let ic_2: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(IC_2_X), from_dec_string(IC_2_Y)))
				.try_into()
				.unwrap();

		//----------END OF VK------------//

		//----------PROOF---------------//
		// sample/proof.json
		let pi_c: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_A_X), from_dec_string(PI_A_Y)))
				.try_into()
				.unwrap();
		let pi_b: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(PI_B_X_C0),
			from_dec_string(PI_B_X_C1),
			from_dec_string(PI_B_Y_C0),
			from_dec_string(PI_B_Y_C1),
		))
			.try_into()
			.unwrap();
		let pi_a: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_C_X), from_dec_string(PI_C_Y)))
				.try_into()
				.unwrap();
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert!(!verify(
			VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1, ic_2] },
			GProof { a: pi_a, b: pi_b, c: pi_c },
			[33.into()].into(),
		)
		.unwrap())
		//--------END OF VERIFICATION---------//
	}

	#[test]
	fn verify_with_incorrect_ic_len() {
		//----------VK------------//
		// sample/verification_key.json
		let alpha: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(ALPHA_X), from_dec_string(ALPHA_Y)))
				.try_into()
				.unwrap();

		let beta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(BETA_X_C0),
			from_dec_string(BETA_X_C1),
			from_dec_string(BETA_Y_C0),
			from_dec_string(BETA_Y_C1),
		))
			.try_into()
			.unwrap();

		let gamma: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(GAMMA_X_C0),
			from_dec_string(GAMMA_X_C1),
			from_dec_string(GAMMA_Y_C0),
			from_dec_string(GAMMA_Y_C1),
		))
			.try_into()
			.unwrap();
		let delta: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(DELTA_X_C0),
			from_dec_string(DELTA_X_C1),
			from_dec_string(DELTA_Y_C0),
			from_dec_string(DELTA_Y_C1),
		))
			.try_into()
			.unwrap();

		let ic_1: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(IC_1_X), from_dec_string(IC_1_Y)))
				.try_into()
				.unwrap();

		//----------END OF VK------------//

		//----------PROOF---------------//
		// sample/proof.json
		let pi_a: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_A_X), from_dec_string(PI_A_Y)))
				.try_into()
				.unwrap();
		let pi_b: G2Affine = (&G2UncompressedBytes::new(
			from_dec_string(PI_B_X_C0),
			from_dec_string(PI_B_X_C1),
			from_dec_string(PI_B_Y_C0),
			from_dec_string(PI_B_Y_C1),
		))
			.try_into()
			.unwrap();
		let pi_c: G1Affine =
			(&G1UncompressedBytes::new(from_dec_string(PI_C_X), from_dec_string(PI_C_Y)))
				.try_into()
				.unwrap();
		//------END OF PROOF-----------//

		//----------VERIFICATION---------------//
		assert_eq!(
			verify(
				VerificationKey { alpha, beta, gamma, delta, ic: vec![ic_1] },
				GProof { a: pi_a, b: pi_b, c: pi_c },
				[33.into()].into(),
			)
			.err()
			.unwrap(),
			VerificationError::InvalidVerificationKey
		)
		//--------END OF VERIFICATION---------//
	}

	fn from_dec_string(number: &str) -> [u8; 48] {
		let mut bytes: [u8; 48] = [0; 48];
		U256::from_dec_str(number).unwrap().to_big_endian(bytes.as_mut_slice());
		bytes
	}
}
