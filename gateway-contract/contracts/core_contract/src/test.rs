#![cfg(test)]

use crate::{Contract, ContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

// --- Helpers for ZK Tests ---

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

fn valid_input(env: &Env) -> BytesN<32> {
    let mut inp = [0u8; 32];
    inp[31] = 0x01; // scalar = 1, well below field prime
    BytesN::from_array(env, &inp)
}

// --- Helpers for Resolver Tests ---

fn setup_test(env: &Env) -> (ContractClient<'_>, BytesN<32>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let commitment = BytesN::from_array(env, &[7u8; 32]);
    let wallet = Address::generate(env);

    (client, commitment, wallet)
}

// --- ZK Proof Verifier Tests ---

#[test]
fn test_verify_proof_valid() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env); // Use helper for registration

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
    let (client, _, _) = setup_test(&env);

    let short_proof = Bytes::from_slice(&env, &[0u8; 64]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(valid_input(&env));

    client.verify_proof(&short_proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_zero_a_point() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let bad_proof = Bytes::from_slice(&env, &[0u8; 256]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(valid_input(&env));

    client.verify_proof(&bad_proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_no_public_inputs() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let proof = valid_proof(&env);
    let inputs: Vec<BytesN<32>> = Vec::new(&env);

    client.verify_proof(&proof, &inputs);
}

#[test]
#[should_panic]
fn test_verify_proof_input_not_in_field() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let proof = valid_proof(&env);
    let bad_input = BytesN::from_array(&env, &[0xffu8; 32]);
    let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
    inputs.push_back(bad_input);

    client.verify_proof(&proof, &inputs);
}

// --- Resolver Logic Tests ---

#[test]
fn test_resolve_returns_none_when_no_memo() {
    let env = Env::default();
    let (client, commitment, wallet) = setup_test(&env);

    client.register_resolver(&commitment, &wallet, &None);

    let (resolved_wallet, memo) = client.resolve(&commitment);
    assert_eq!(resolved_wallet, wallet);
    assert_eq!(memo, None);
}

#[test]
fn test_set_memo_and_resolve_flow() {
    let env = Env::default();
    let (client, commitment, wallet) = setup_test(&env);

    client.register_resolver(&commitment, &wallet, &None);
    client.set_memo(&commitment, &4242u64);

    let (resolved_wallet, memo) = client.resolve(&commitment);
    assert_eq!(resolved_wallet, wallet);
    assert_eq!(memo, Some(4242u64));
}