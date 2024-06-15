use std::io::BufReader;
use crate::card::{create_circuit, empty_circuit,
	generate_setup_params, generate_keys,
	generate_proof, verify, create_public_inputs};
use halo2_proofs::{
	poly::commitment::Params,
	pasta::{Fp, EqAffine},
	plonk::keygen_vk
};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

fn copy_vec_to_u8arr(v: &Vec<u8>) -> Uint8Array {
	let u8_arr = Uint8Array::new_with_length(v.len() as u32);
	u8_arr.copy_from(v);
	u8_arr
}

#[wasm_bindgen]
pub fn setup_params(k: u32) -> Uint8Array {
	log("running setup");

	// Generate setup params
	let params = generate_setup_params(k);
	let mut buf = vec![];
	params.write(&mut buf).expect("Can write params");

	copy_vec_to_u8arr(&buf)
} 

#[wasm_bindgen]
pub fn proof_generate( qty: u64, suite: u64, rank: u64,
	param_bytes: &[u8]) -> Uint8Array {

	log("proving..");

	// Read params
	let params = Params::<EqAffine>::read(
					&mut BufReader::new(param_bytes)).
					expect("Failed to read params");

	// Create public inputs
	let public_inputs = create_public_inputs(qty, suite, rank);

	// Generate proving key
	let empty_circuit = empty_circuit();
	let (pk, _vk) = generate_keys::<Fp>(&params, &empty_circuit);

	// Generate proof
	let card_circuit = create_circuit(qty, suite, rank);
	let proof = generate_proof(&params, &pk, card_circuit,
					(&public_inputs.0, &public_inputs.1,
					 &public_inputs.2));

	copy_vec_to_u8arr(&proof)
}

#[wasm_bindgen]
pub fn proof_verify(param_bytes: &[u8], qty: u64, suite: u64,
			rank: u64, proof: &[u8]) -> bool {

	log("verifying..");

	let params = Params::<EqAffine>::read(
					&mut BufReader::new(param_bytes)).
					expect("Failed to read params");

	// Generate verifying key
	let empty_circuit = empty_circuit();
	let vk = keygen_vk(&params, &empty_circuit)
		.expect("Failed to generate verifying key");

	// Create public inputs
	let public_inputs = create_public_inputs(qty, suite, rank);

	// Trasform proof to vector
	let proof_vec = proof.to_vec();

	// Verify the proof and public input
    let ret = verify(&params, &vk, (&public_inputs.0, &public_inputs.1,
                        &public_inputs.2), proof_vec);

    match ret {
    	Err(_) => false,
    	_ => true,
    }
}
	