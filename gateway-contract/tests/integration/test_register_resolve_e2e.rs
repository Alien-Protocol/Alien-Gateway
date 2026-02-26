// End-to-End Integration Test: Register → Resolve Flow

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use alien_gateway::ContractClient;

#[test]
fn test_register_resolve_e2e() {
    // 1. Simulate off-chain ZK proof generation (mocked for test)
    let env = Env::default();
    let contract_id = env.register(alien_gateway::Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Example username and wallet address
    let username = "amar";
    let wallet = Address::generate(&env);

    // Simulate off-chain: hash username to get commitment (mocked as [1u8; 32])
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    // 2. Register the username (with proof, here we just call register_resolver)
    client.register_resolver(&commitment, &wallet, &None);

    // 3. Resolve the username (by commitment)
    let result = client.resolve(&commitment);

    // 4. Assert the resolved wallet matches
    assert_eq!(result.wallet, wallet);
    assert!(result.memo.is_none());
}
