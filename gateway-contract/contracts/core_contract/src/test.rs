#![cfg(test)]

use crate::{Contract, ContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

fn setup_test(env: &Env) -> (ContractClient<'_>, BytesN<32>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let commitment = BytesN::from_array(env, &[7u8; 32]);
    let wallet = Address::generate(env);

    (client, commitment, wallet)
}

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

#[test]
fn test_privacy_mode_resolve_shielded() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    let wallet = Address::generate(&env);

    // Mock registration (owner)
    env.as_contract(&contract_id, || {
        let key = crate::registration::DataKey::Commitment(commitment.clone());
        env.storage().persistent().set(&key, &wallet);
    });

    client.register_resolver(&commitment, &wallet, &None);

    // Default should be Normal
    assert!(client.get_privacy_mode(&commitment) == crate::types::PrivacyMode::Normal);
    let (resolved, _) = client.resolve(&commitment);
    assert_eq!(resolved, wallet);

    // Set to Private
    env.mock_all_auths();
    client.set_privacy_mode(&commitment, &crate::types::PrivacyMode::Private);
    
    // Resolve should return contract address (shielded)
    let (resolved_shielded, _) = client.resolve(&commitment);
    assert_eq!(resolved_shielded, contract_id);
}

#[test]
fn test_privacy_mode_transition_back_to_normal() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let commitment = BytesN::from_array(&env, &[2u8; 32]);
    let wallet = Address::generate(&env);

    // Mock registration (owner)
    env.as_contract(&contract_id, || {
        let key = crate::registration::DataKey::Commitment(commitment.clone());
        env.storage().persistent().set(&key, &wallet);
    });

    client.register_resolver(&commitment, &wallet, &None);
    env.mock_all_auths();

    // Set to Private
    client.set_privacy_mode(&commitment, &crate::types::PrivacyMode::Private);
    let (res_p, _) = client.resolve(&commitment);
    assert_eq!(res_p, contract_id);

    // Set back to Normal
    client.set_privacy_mode(&commitment, &crate::types::PrivacyMode::Normal);
    let (res_n, _) = client.resolve(&commitment);
    assert_eq!(res_n, wallet);
}
