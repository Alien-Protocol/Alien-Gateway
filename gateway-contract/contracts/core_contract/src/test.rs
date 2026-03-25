#![cfg(test)]

use crate::errors::ContractError;
use crate::types::{Proof, PublicSignals};
use crate::{CoreContract, CoreContractClient};
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{
    contract, contractimpl, contracttype, map, Address, BytesN, Env, Error, IntoVal, Map, Symbol,
    Val,
};

#[contract]
struct MockVerifierContract;

#[contracttype]
#[derive(Clone)]
enum MockVerifierDataKey {
    ShouldVerify,
}

#[contractimpl]
impl MockVerifierContract {
    pub fn set_should_verify(env: Env, should_verify: bool) {
        env.storage()
            .instance()
            .set(&MockVerifierDataKey::ShouldVerify, &should_verify);
    }

    pub fn verify_proof(env: Env, proof: Proof, public_signals: PublicSignals) -> bool {
        let should_verify = env
            .storage()
            .instance()
            .get::<MockVerifierDataKey, bool>(&MockVerifierDataKey::ShouldVerify)
            .unwrap_or(true);

        should_verify
            && proof.a == public_signals.old_root
            && proof.b == public_signals.new_root
            && proof.c == public_signals.commitment
    }
}

fn bytes(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn offchain_register_fixture(env: &Env) -> (Proof, PublicSignals) {
    let old_root = bytes(env, 0);
    let new_root = bytes(env, 42);
    let commitment = bytes(env, 7);

    (
        Proof {
            a: old_root.clone(),
            b: new_root.clone(),
            c: commitment.clone(),
        },
        PublicSignals {
            old_root,
            new_root,
            commitment,
        },
    )
}

fn setup(env: &Env) -> (Address, CoreContractClient<'_>, Address) {
    let verifier_id = env.register(MockVerifierContract, ());
    let verifier_client = MockVerifierContractClient::new(env, &verifier_id);
    verifier_client.set_should_verify(&true);

    let contract_id = env.register(CoreContract, ());
    let client = CoreContractClient::new(env, &contract_id);
    client.init(&verifier_id, &bytes(env, 0));

    (contract_id, client, verifier_id)
}

fn assert_submit_error(
    result: Result<
        Result<(), soroban_sdk::ConversionError>,
        Result<Error, soroban_sdk::InvokeError>,
    >,
    expected: ContractError,
) {
    assert_eq!(result, Err(Ok(expected.into())));
}

#[test]
fn submit_proof_succeeds_and_updates_state() {
    let env = Env::default();
    let (contract_id, client, _) = setup(&env);
    let (proof, public_signals) = offchain_register_fixture(&env);

    client.submit_proof(&proof, &public_signals);

    assert_eq!(client.get_root(), Some(public_signals.new_root.clone()));
    assert!(client.has_commitment(&public_signals.commitment));

    let expected_event_data: Map<Symbol, Val> = map![
        &env,
        (
            Symbol::new(&env, "commitment"),
            public_signals.commitment.clone().into_val(&env)
        )
    ];
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "username_registered"),).into_val(&env),
                expected_event_data.into_val(&env),
            )
        ]
    );
}

#[test]
fn invalid_proof_is_rejected() {
    let env = Env::default();
    let (_, client, verifier_id) = setup(&env);
    let verifier_client = MockVerifierContractClient::new(&env, &verifier_id);
    verifier_client.set_should_verify(&false);

    let (proof, public_signals) = offchain_register_fixture(&env);
    let result = client.try_submit_proof(&proof, &public_signals);

    assert_submit_error(result, ContractError::InvalidProof);
    assert!(!client.has_commitment(&public_signals.commitment));
    assert_eq!(client.get_root(), Some(public_signals.old_root));
}

#[test]
fn stale_root_is_rejected() {
    let env = Env::default();
    let (_, client, _) = setup(&env);
    let (proof, mut public_signals) = offchain_register_fixture(&env);
    public_signals.old_root = bytes(&env, 1);

    let result = client.try_submit_proof(&proof, &public_signals);

    assert_submit_error(result, ContractError::RootMismatch);
    assert!(!client.has_commitment(&public_signals.commitment));
    assert_eq!(client.get_root(), Some(bytes(&env, 0)));
}

#[test]
fn duplicate_commitment_is_rejected() {
    let env = Env::default();
    let (_, client, _) = setup(&env);
    let (proof, public_signals) = offchain_register_fixture(&env);

    client.submit_proof(&proof, &public_signals);

    let duplicate_result = client.try_submit_proof(&proof, &public_signals);

    assert_submit_error(duplicate_result, ContractError::DuplicateCommitment);
    assert_eq!(client.get_root(), Some(public_signals.new_root));
}
