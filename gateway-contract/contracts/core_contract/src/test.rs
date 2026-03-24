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
