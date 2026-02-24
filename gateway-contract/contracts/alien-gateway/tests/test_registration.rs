use alien_gateway::{Contract, Registration};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

#[test]
fn test_successful_registration() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());

    // Mock invoker address
    let user = Address::generate(&env);
    env.mock_all_auths();

    // Generate a test commitment (32 bytes)
    let commitment: BytesN<32> = BytesN::random(&env);

    // Register the commitment
    env.as_contract(&contract_id, || {
        Registration::register(env.clone(), user.clone(), commitment.clone());
    });

    // Verify commitment is mapped to the user
    let owner = env.as_contract(&contract_id, || {
        Registration::get_owner(env.clone(), commitment.clone())
    });

    assert_eq!(owner, Some(user));
}

#[test]
#[should_panic(expected = "Commitment already registered")]
fn test_duplicate_commitment_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    env.mock_all_auths();

    let commitment: BytesN<32> = BytesN::random(&env);

    // First registration should succeed
    env.as_contract(&contract_id, || {
        Registration::register(env.clone(), user1.clone(), commitment.clone());
    });

    // Second registration with same commitment should panic
    env.as_contract(&contract_id, || {
        Registration::register(env.clone(), user2, commitment);
    });
}

#[test]
fn test_multiple_users_different_commitments() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    env.mock_all_auths();

    let commitment1: BytesN<32> = BytesN::random(&env);
    let commitment2: BytesN<32> = BytesN::random(&env);

    // Register first user with first commitment
    env.as_contract(&contract_id, || {
        Registration::register(env.clone(), user1.clone(), commitment1.clone());
    });

    // Register second user with second commitment
    env.as_contract(&contract_id, || {
        Registration::register(env.clone(), user2.clone(), commitment2.clone());
    });

    // Verify both commitments are mapped correctly
    env.as_contract(&contract_id, || {
        let owner1 = Registration::get_owner(env.clone(), commitment1);
        let owner2 = Registration::get_owner(env.clone(), commitment2);

        assert_eq!(owner1, Some(user1));
        assert_eq!(owner2, Some(user2));
    });
}

#[test]
fn test_get_owner_nonexistent_commitment() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());

    let commitment: BytesN<32> = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        let owner = Registration::get_owner(env.clone(), commitment);
        assert_eq!(owner, None);
    });
}
