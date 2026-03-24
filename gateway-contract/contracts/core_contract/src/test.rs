#![cfg(test)]

use super::*;
use crate::zk_verifiers::Groth16Proof;
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env};

fn create_test_proof(env: &Env) -> Groth16Proof {
    Groth16Proof {
        pi_a: vec![env, BytesN::from_array(env, &[0; 32])],
        pi_b: vec![env, vec![env, BytesN::from_array(env, &[0; 32])]],
        pi_c: vec![env, BytesN::from_array(env, &[0; 32])],
    }
}

#[test]
fn test_registration_success() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let wallet = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1; 32]);
    let proof = create_test_proof(&env);

    client.register_resolver(&commitment, &wallet, &None, &proof);
    let (resolved_wallet, _) = client.resolve(&commitment);
    assert_eq!(resolved_wallet, wallet);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_registration_invalid_proof() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let wallet = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1; 32]);

    let invalid_proof = Groth16Proof {
        pi_a: vec![&env],
        pi_b: vec![&env],
        pi_c: vec![&env],
    };

    client.register_resolver(&commitment, &wallet, &None, &invalid_proof);
}
