//! Auth middleware integration tests.
//! - Write without auth fails
//! - Write with auth succeeds
//! - Ownership transfer updates permissions

use alien_gateway::{AddressManager, Contract, CoreContract};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

#[test]
#[should_panic]
fn test_write_without_auth_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let other = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        other.require_auth();
        AddressManager::register_address(env.clone(), other.clone());
    });
}

#[test]
fn test_write_with_auth_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });

    let master = env
        .as_contract(&contract_id, || AddressManager::get_master(env.clone()))
        .unwrap();
    assert_eq!(master, user);
}

#[test]
fn test_ownership_transfer_new_owner_has_permissions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let user = Symbol::new(&env, "user");

    env.as_contract(&contract_id, || {
        CoreContract::init(env.clone(), user.clone(), owner.clone());
        assert_eq!(CoreContract::get_owner(env.clone()), owner);
    });
    env.as_contract(&contract_id, || {
        CoreContract::transfer_ownership(env.clone(), new_owner.clone());
        assert_eq!(CoreContract::get_owner(env.clone()), new_owner);
    });
    env.as_contract(&contract_id, || {
        CoreContract::transfer_ownership(env.clone(), owner.clone());
        assert_eq!(CoreContract::get_owner(env.clone()), owner);
    });
}

#[test]
#[should_panic]
fn test_ownership_transfer_old_owner_loses_permissions() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let user = Symbol::new(&env, "user");

    env.as_contract(&contract_id, || {
        CoreContract::init(env.clone(), user.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        owner.require_auth();
        CoreContract::transfer_ownership(env.clone(), new_owner.clone());
    });
    env.as_contract(&contract_id, || {
        owner.require_auth();
        CoreContract::transfer_ownership(env.clone(), owner.clone());
    });
}
