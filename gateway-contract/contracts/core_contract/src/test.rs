#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as AddressUtil, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal,
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
