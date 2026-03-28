#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::Address as AddressUtil,
    Address, Env,
};

use crate::contract_core::auth;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Register and deploy the contract, returning the Env and its contract address.
fn setup() -> (Env, Address) {
    let env = Env::default();
    let contract_id = env.register(crate::Contract, ());
    (env, contract_id)
}

/// Seed instance storage with an owner (simulates a successful `init`).
fn seed_owner(env: &Env, contract_id: &Address, owner: &Address) {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .set(&soroban_sdk::symbol_short!("Owner"), owner);
    });
}

// ─── tests ──────────────────────────────────────────────────────────────────

/// The owner is stored and calls a write function — no panic expected.
#[test]
fn test_require_owner_succeeds_for_owner() {
    let (env, contract_id) = setup();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    seed_owner(&env, &contract_id, &owner);

    // require_owner should not panic when called by the stored owner with mocked auth.
    env.as_contract(&contract_id, || {
        auth::require_owner(&env);
    });
}

/// A non-owner tries to call a write function — expect a Soroban auth panic.
#[test]
#[should_panic]
fn test_require_owner_panics_for_non_owner() {
    let (env, contract_id) = setup();
    // Do NOT call mock_all_auths — auth checks will fail for any caller that is
    // not explicitly authorized.

    let owner = Address::generate(&env);
    seed_owner(&env, &contract_id, &owner);

    // Run require_owner without any auth mocking.  Soroban will panic because
    // `owner.require_auth()` cannot be satisfied.
    env.as_contract(&contract_id, || {
        auth::require_owner(&env);
    });
}

/// Calling require_owner before init (no Owner in storage) must panic with
/// the message "Contract not initialized".
#[test]
#[should_panic(expected = "Contract not initialized")]
fn test_require_owner_panics_if_uninitialized() {
    let (env, contract_id) = setup();
    env.mock_all_auths();

    // No seed_owner call — instance storage has no Owner key.
    env.as_contract(&contract_id, || {
        auth::require_owner(&env);
    });
}

// ============================================================================
// initialize / get_contract_owner tests  (Issue #187)
// ============================================================================

#[test]
fn test_initialize_stores_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let owner = Address::generate(&env);
    client.initialize(&owner);

    assert_eq!(client.get_contract_owner(), owner);
}

#[test]
fn test_initialize_emits_init_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);

    let owner = Address::generate(&env);
    client.initialize(&owner);

    let events = env.events().all();
    let has_init_event = events.iter().any(|(c, _, _)| c == contract_id);
    assert!(has_init_event);
}

#[test]
fn test_initialize_double_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let owner = Address::generate(&env);
    client.initialize(&owner);

    let result = client.try_initialize(&owner);
    assert!(result.is_err());
}

#[test]
fn test_get_contract_owner_before_init_panics() {
    let env = Env::default();
    let (_, client) = setup(&env);

    let result = client.try_get_contract_owner();
    assert!(result.is_err());
}

// ============================================================================
// add_shielded_address / get_shielded_address / is_shielded tests  (Issue #193)
// ============================================================================

#[test]
fn test_add_shielded_address_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let owner = Address::generate(&env);
    let hash = commitment(&env, 80);
    let addr_commitment = BytesN::from_array(&env, &[0xAAu8; 32]);

    client.register(&owner, &hash);
    client.add_shielded_address(&owner, &hash, &addr_commitment);

    assert_eq!(client.get_shielded_address(&hash), Some(addr_commitment));
    assert!(client.is_shielded(&hash));
}

#[test]
fn test_is_shielded_returns_false_when_not_set() {
    let env = Env::default();
    let (_, client) = setup(&env);

    let hash = commitment(&env, 81);
    assert!(!client.is_shielded(&hash));
}

#[test]
fn test_add_shielded_address_overwrite_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let owner = Address::generate(&env);
    let hash = commitment(&env, 82);
    let first = BytesN::from_array(&env, &[0x11u8; 32]);
    let second = BytesN::from_array(&env, &[0x22u8; 32]);

    client.register(&owner, &hash);
    client.add_shielded_address(&owner, &hash, &first);
    client.add_shielded_address(&owner, &hash, &second);

    assert_eq!(client.get_shielded_address(&hash), Some(second));
}

#[test]
fn test_add_shielded_address_non_owner_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let hash = commitment(&env, 83);
    let addr_commitment = BytesN::from_array(&env, &[0xBBu8; 32]);

    client.register(&owner, &hash);

    env.mock_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "add_shielded_address",
            args: (&attacker, &hash, &addr_commitment).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_add_shielded_address(&attacker, &hash, &addr_commitment);
    assert!(result.is_err());
}

#[test]
fn test_add_shielded_address_unregistered_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let caller = Address::generate(&env);
    let hash = commitment(&env, 84);
    let addr_commitment = BytesN::from_array(&env, &[0xCCu8; 32]);

    let result = client.try_add_shielded_address(&caller, &hash, &addr_commitment);
    assert!(result.is_err());
}
