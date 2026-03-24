#![cfg(test)]
use crate::Contract;
use crate::ContractClient;
use soroban_sdk::{Bytes, BytesN, Env, Vec};

// Helper: build a valid 256-byte proof (non-zero A and C points)
fn valid_proof(env: &Env) -> Bytes {
    let mut buf = [0u8; 256];
    // A.x — non-zero
    buf[0] = 0x1a;
    buf[1] = 0x2b;
    // A.y
    buf[32] = 0x3c;
    // B.x0 — non-zero
    buf[64] = 0x4d;
    // C.x — non-zero
    buf[192] = 0x5e;
    Bytes::from_slice(env, &buf)
}

// Helper: build a valid 32-byte public input (below BN254 prime)
fn valid_input(env: &Env) -> BytesN<32> {
    let mut inp = [0u8; 32];
    inp[31] = 0x01; // scalar = 1, well below field prime
    BytesN::from_array(env, &inp)
}

#[test]
fn test_verify_proof_valid() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let proof = valid_proof(&env);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(valid_input(&env));

    let result = client.verify_proof(&proof, &inputs);
    assert!(result, "valid proof should be accepted");
}

#[test]
#[should_panic]
fn test_verify_proof_wrong_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Only 64 bytes — should panic with InvalidProofLength
    let short_proof = Bytes::from_slice(&env, &[0u8; 64]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(valid_input(&env));

    client.verify_proof(&short_proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_zero_a_point() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // All-zero proof — A.x is zero so G1 validation fails → InvalidProof
    let bad_proof = Bytes::from_slice(&env, &[0u8; 256]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(valid_input(&env));

    client.verify_proof(&bad_proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_no_public_inputs() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let proof = valid_proof(&env);
    let inputs: Vec<BytesN<32>> = Vec::new(&env); // empty — wrong count

    client.verify_proof(&proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_input_not_in_field() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let proof = valid_proof(&env);
    // Input larger than BN254 prime (all 0xFF bytes)
    let bad_input = BytesN::from_array(&env, &[0xffu8; 32]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(bad_input);

    client.verify_proof(&proof, &inputs);
}
