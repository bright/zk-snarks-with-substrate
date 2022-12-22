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
		let vk = prepare_vk_json("groth16", "bls12381", None);
		assert_ok!(ZKSnarks::setup_verification(
			RuntimeOrigin::none(),
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}
#[test]
fn test_not_supported_vk_curve() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bn128", None);
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				prepare_correct_public_inputs_json().as_bytes().into(),
				vk.as_bytes().into()
			),
			Error::<Test>::NotSupportedCurve
		);
		let events = zk_events();
		assert_eq!(events.len(), 0);
	});
}

#[test]
fn test_not_supported_vk_protocol() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("-", "bls12381", None);
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				prepare_correct_public_inputs_json().as_bytes().into(),
				vk.as_bytes().into()
			),
			Error::<Test>::NotSupportedProtocol
		);
		let events = zk_events();
		assert_eq!(events.len(), 0);
	});
}

#[test]
fn test_too_long_verification_key() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				prepare_correct_public_inputs_json().as_bytes().into(),
				vec![0; (<Test as Config>::MaxVerificationKeyLength::get() + 1) as usize]
			),
			Error::<Test>::TooLongVerificationKey
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_too_long_public_inputs() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				vec![0; (<Test as Config>::MaxPublicInputsLength::get() + 1) as usize],
				prepare_vk_json("groth16", "bls12381", None).as_bytes().into()
			),
			Error::<Test>::TooLongPublicInputs
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_public_inputs_mismatch() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::setup_verification(
				RuntimeOrigin::none(),
				prepare_empty_public_inputs_json().as_bytes().into(),
				prepare_vk_json("groth16", "bls12381", None).as_bytes().into()
			),
			Error::<Test>::PublicInputsMismatch
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_too_long_proof() {
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
fn test_not_supported_proof_protocol() {
	let proof = prepare_proof_json("-", "bls12381", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()),
			Error::<Test>::NotSupportedProtocol
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_not_supported_proof_curve() {
	let proof = prepare_proof_json("groth16", "bn128", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()),
			Error::<Test>::NotSupportedCurve
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_empty_proof() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), Vec::new()),
			Error::<Test>::ProofIsEmpty
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verify_without_verification_key() {
	let proof = prepare_proof_json("groth16", "bls12381", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()),
			Error::<Test>::VerificationKeyIsNotSet
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verification_success() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKSnarks::setup_verification(
			RuntimeOrigin::none(),
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_ok!(ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(events[2], Event::<Test>::VerificationSuccess);
	});
}

#[test]
fn test_verification_failed() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKSnarks::setup_verification(
			RuntimeOrigin::none(),
			prepare_incorrect_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_ok!(ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(events[2], Event::<Test>::VerificationFailed);
	});
}

#[test]
fn test_could_not_create_proof() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", Some("12".to_owned()));

		assert_ok!(ZKSnarks::setup_verification(
			RuntimeOrigin::none(),
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()),
			Error::<Test>::ProofCreationError
		);

		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}

#[test]
fn test_could_not_create_verification_key() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", Some("12".to_owned()));
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKSnarks::setup_verification(
			RuntimeOrigin::none(),
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_err!(
			ZKSnarks::verify(RuntimeOrigin::none(), proof.as_bytes().into()),
			Error::<Test>::VerificationKeyCreationError
		);

		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}

fn prepare_correct_public_inputs_json() -> String {
	r#"[
 "33"
]"#
	.to_owned()
}

fn prepare_incorrect_public_inputs_json() -> String {
	r#"[
 "3"
]"#
	.to_owned()
}

fn prepare_empty_public_inputs_json() -> String {
	r#"[
]"#
	.to_owned()
}

fn prepare_vk_json(protocol: &str, curve: &str, alpha_x: Option<String>) -> String {
	let alpha_x = alpha_x.unwrap_or_else(|| "2635983656263320256511463995836413167331869092392943593306076905516259749312747842295447349507189592731785901862558".to_owned());
	let vk_template = r#"{
 "protocol": "<protocol>",
 "curve": "<curve>",
 "nPublic": 1,
 "vk_alpha_1": [
  "<alpha_x>",
  "743892996456702519498029594549937288641619275055957975879157306988929970626325326222697609972550552691064908651931",
  "1"
 ],
 "vk_beta_2": [
  [
   "1296094501238138520689116246487755613076576267512760150298482409401507546730337296489670510433482825207790998397346",
   "3467549840163329914429787393326495235851806074050417925094845001935796859739058829480949031354270816778136382040361"
  ],
  [
   "3403410200913851046378881164751590587066009874691619938225021193334979700147466129997648606538377099567064346931273",
   "3804847074485539411700684267722735363688167108429634491643293100788171321105199556340902873511607444652008144844173"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_gamma_2": [
  [
   "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160",
   "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758"
  ],
  [
   "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905",
   "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_delta_2": [
  [
   "3284950120787447527021651154232749836311526699432807747366843303661566424019671125328694916562846460813145647040459",
   "3218306808275776807419693666072599084905639169987324420818366627509865827220976650759812930713956208246000627242485"
  ],
  [
   "3945290144137392347873751586031392152201459997902585432454016489727689337013866944382877542451368688652560743518350",
   "3505020872425170466568261366418107787485649574216477007429328593907934456720034754142706045279478244784882964969099"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_alphabeta_12": [
  [
   [
    "2875682627859788046787727046323207700818211438271057184016590533515641699292997693457599620936708894947724425715231",
    "1238832727101571020174962081840437018121939792461931445079462232794726384752259447129206595688503509959353794284793"
   ],
   [
    "1142295393527745936520465586775444768688364741373237930118445421796520414741916849824256960449892474465014692624756",
    "2180077006016464788050801868062734927906767334913187269719534809313436039282935753136702491423916116028147695108113"
   ],
   [
    "581912189975592585217934845255593126879157415518933223520266217690258707840190591176174259119561150786976644927862",
    "1496521185256234033198775390415811847166244093737149241712223242576005202124661827966889067451826629794876020037891"
   ]
  ],
  [
   [
    "968778761326544533347894440852946317832878172436078056438728764792716948106777133186592741979864246862480026990714",
    "3286237875677076419439678035167386721716851772116127087476697302808027553397980022598381369816552966804476744614726"
   ],
   [
    "703046133019192877150497098682775062870944581080811653558417167836034365682308629278579084636589495681129838804552",
    "3120651492951743750811126470515331662411558962596191689455151216422711804034698152168980665082907679235009776566592"
   ],
   [
    "3093035865177537484265129293484086930964325066660842965056946750881983192007730606218463861804151316907199193750598",
    "2217088332657331378025998358211322741524769834682072728928845130805944349335376146743275401044544953792401446016391"
   ]
  ]
 ],
 "IC": [
  [
   "3759794041598018594287463133849401670165044879836734797942436987012929463856866218164906521458646350224910548839839",
   "3238512100593065266229132824040292706800754984648723917955334599968665051423411534393542324672325614522917210582797",
   "1"
  ],
  [
   "3305881491744710205856868316456114914540772066725994230747514104922282269209779243587827394909802115252764372519712",
   "2462443929524735084767395208674598757462820081953985438437610428598624587728712969052746628125821805697605346885091",
   "1"
  ]
 ]
}"#;
	vk_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<alpha_x>", &alpha_x)
}

fn prepare_proof_json(protocol: &str, curve: &str, pi_a_x: Option<String>) -> String {
	let pi_a_x = pi_a_x.unwrap_or_else(|| "2820173869801000183955769496344276101575675010174203082588560105436284422640780128231242184109173767085197647834267".to_owned());
	let proof_template = r#"{
 "pi_a": [
  "<pi_a_x>",
  "1152541093585973172499551859168528642628429504007613830168996825879806250289422935864437193085184388469171892221011",
  "1"
 ],
 "pi_b": [
  [
   "54413090665354594353317256815335793052197307111690011609872716599840507808706991989403605000342095250665180513594",
   "3343285764332210309442703216841128605678475246673133285314301861643378387001264758819444434632415207857557469906035"
  ],
  [
   "262180403851765105493218367619507205740764669171746348153545090105487398261554724750259283942935411519021270362742",
   "3777303780170739988308854254585940898119682705621212814969008224084499326863117961704608873229314936725151172212883"
  ],
  [
   "1",
   "0"
  ]
 ],
 "pi_c": [
  "3006923877346016048391409264528383002939756547318806158402407618139299715778986391175418348881376388499383266389442",
  "1307513151230758506579817970515482216448699470263630520204374492458260823057506418477833081567163581258564509876945",
  "1"
 ],
 "protocol": "<protocol>",
 "curve": "<curve>"
}"#;

	proof_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<pi_a_x>", &pi_a_x)
}
