#[cfg(not(target_family = "wasm"))]

fn main() {
    use card::card::{empty_circuit, create_circuit, generate_setup_params,
                    generate_keys, run_mock_prover, generate_proof, verify,
                    create_public_inputs};

    // Size of the circuit
    let k = 5;

    // Private input to generate a proof with
    let qty = 1 as u64;
    let suite = 1 as u64;
    let rank = 2 as u64;
    let public_inputs = create_public_inputs(qty, suite, rank);

    // Create the circuit
    let card_circuit = create_circuit(qty, suite, rank);

    // Run mock prover
    run_mock_prover(k, &card_circuit, public_inputs);
}