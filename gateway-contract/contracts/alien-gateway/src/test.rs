#![cfg(test)]

use crate::{Contract, ContractClient};
use soroban_sdk::testutils::Address as AddressTestUtils;
use soroban_sdk::{Address, BytesN, Env};

#[test]
fn test_resolve_found_without_memo() {
    let env = Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    let wallet = <Address as AddressTestUtils>::generate(&env);

    client.register_resolver(&commitment, &wallet, &None);

    let result = client.resolve(&commitment);

    assert_eq!(result.wallet, wallet);
    assert!(result.memo.is_none());
}

#[test]
fn test_resolve_found_with_memo() {
    let env = Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let commitment = BytesN::from_array(&env, &[2u8; 32]);

    let wallet = <Address as AddressTestUtils>::generate(&env);

    client.register_resolver(&commitment, &wallet, &Some(12345));

    let result = client.resolve(&commitment);

    assert_eq!(result.wallet, wallet);
    assert_eq!(result.memo.unwrap(), 12345);
}
#[test]
#[should_panic]
fn test_resolve_not_found() {
    let env = Env::default();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let commitment = BytesN::from_array(&env, &[9u8; 32]);

    client.resolve(&commitment);
}
