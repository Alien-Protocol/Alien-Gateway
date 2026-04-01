use crate::{Contract, ContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

#[test]
fn test_register_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client.register(&caller, &commitment);

    let owner = client.get_owner(&commitment);
    assert_eq!(owner, Some(caller));
}

#[test]
#[should_panic(expected = "Commitment already registered")]
fn test_register_duplicate_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[2u8; 32]);

    client.register(&caller, &commitment);
    client.register(&caller, &commitment);
}

#[test]
fn test_get_owner_returns_owner_after_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[3u8; 32]);

    client.register(&caller, &commitment);

    let owner = client.get_owner(&commitment);
    assert_eq!(owner, Some(caller));
}

#[test]
fn test_get_owner_returns_none_for_unknown() {
    let env = Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let unknown_commitment = BytesN::from_array(&env, &[99u8; 32]);

    let owner = client.get_owner(&unknown_commitment);
    assert_eq!(owner, None);
}
