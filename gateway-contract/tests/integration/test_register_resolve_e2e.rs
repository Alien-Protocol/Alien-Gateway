// End-to-End Integration Test: Register → Resolve Flow

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use alien_gateway::ContractClient;

#[test]
fn test_register_resolve_e2e() {
    let env = Env::default();
    let contract_id = env.register(alien_gateway::Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let username = "amar";
    let wallet = Address::generate(&env);

    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client.register_resolver(&commitment, &wallet, &None);

    let result = client.resolve(&commitment);

    assert_eq!(result.wallet, wallet);
    assert!(result.memo.is_none());
}
