#[cfg(not(target_family = "wasm"))]

use halo2_proofs::pasta::{Fp};

fn main() {
    use card::card::{empty_circuit, create_circuit, generate_setup_params,
                    generate_keys, run_mock_prover, generate_proof, verify,
                    create_public_inputs};

    // Size of the circuit
    let k = 5;

    // Private input to generate a proof with
    let qty = 3 as u64;
    let suite = 3 as u64;
    let rank = 11 as u64;
    let public_inputs = create_public_inputs(qty, suite, rank);

    // Create the circuit
    let card_circuit = create_circuit(qty, suite, rank);

    // Run mock prover    
    run_mock_prover(k, &card_circuit, (&public_inputs.0,
                        &public_inputs.1, &public_inputs.2));

    // Generate setup parameters
    let params = generate_setup_params(k);

    // Generate proving and verification keys
    let empty_circuit = empty_circuit();
    let (pk, vk) = generate_keys::<Fp>(&params, &empty_circuit);

    // Generate proof
    let proof = generate_proof(&params, &pk, card_circuit, (&public_inputs.0,
                        &public_inputs.1, &public_inputs.2));

    let verify = verify(&params, &vk, (&public_inputs.0, &public_inputs.1,
                        &public_inputs.2), proof);

    println!("Verify results:{:?}", verify);
}