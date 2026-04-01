use crate::{Contract, ContractClient};
use super::setup_with_owner;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String};

#[test]
fn test_set_master_stellar_address_success() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let stellar_address = String::from_str(&env, "GXXXXXXXXXXXXXXX");

    client.register_address(&owner, &owner);
    client.set_master_stellar_address(&owner, &commitment, &stellar_address);

    let master = client.get_master();
    assert_eq!(master, Some(stellar_address));
}

#[test]
#[should_panic(expected = "Address not registered")]
fn test_set_master_stellar_address_not_registered() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let stellar_address = String::from_str(&env, "GXXXXXXXXXXXXXXX");

    client.set_master_stellar_address(&owner, &commitment, &stellar_address);
}

#[test]
#[should_panic(expected = "Not owner")]
fn test_set_master_stellar_address_non_owner() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let non_owner = Address::generate(&env);
    let stellar_address = String::from_str(&env, "GXXXXXXXXXXXXXXX");

    client.register_address(&owner, &owner);
    client.set_master_stellar_address(&non_owner, &commitment, &stellar_address);
}

#[test]
fn test_add_stellar_address_success() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let stellar_address = String::from_str(&env, "GYYYYYYYYYYYYYYYY");

    client.add_stellar_address(&owner, &commitment, &stellar_address);

    let stored = client.get_stellar_address(&commitment);
    assert_eq!(stored, Some(stellar_address));
}

#[test]
#[should_panic(expected = "Address already exists")]
fn test_add_stellar_address_duplicate_rejection() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let stellar_address = String::from_str(&env, "GZZZZZZZZZZZZZZZZ");

    client.add_stellar_address(&owner, &commitment, &stellar_address);
    client.add_stellar_address(&owner, &commitment, &stellar_address);
}

#[test]
#[should_panic(expected = "Not owner")]
fn test_add_stellar_address_non_owner_rejection() {
    let env = Env::default();
    let (client, _owner, commitment) = setup_with_owner(&env);
    let non_owner = Address::generate(&env);
    let stellar_address = String::from_str(&env, "GAAAAAAAAAAAAAAAA");

    client.add_stellar_address(&non_owner, &commitment, &stellar_address);
}

#[test]
fn test_register_address_success() {
    let env = Env::default();
    let (client, owner, _commitment) = setup_with_owner(&env);
    let address_to_register = Address::generate(&env);

    client.register_address(&owner, &address_to_register);

    let is_registered = client.is_address_registered(&address_to_register);
    assert!(is_registered);
}

#[test]
#[should_panic(expected = "Not owner")]
fn test_register_address_non_owner_rejection() {
    let env = Env::default();
    let (client, _owner, _commitment) = setup_with_owner(&env);
    let non_owner = Address::generate(&env);
    let address_to_register = Address::generate(&env);

    client.register_address(&non_owner, &address_to_register);
}

#[test]
fn test_get_master_returns_correct_address() {
    let env = Env::default();
    let (client, owner, commitment) = setup_with_owner(&env);
    let stellar_address = String::from_str(&env, "GBBBBBBBBBBBBBBB");

    client.register_address(&owner, &owner);
    client.set_master_stellar_address(&owner, &commitment, &stellar_address);

    let master = client.get_master();
    assert_eq!(master, Some(stellar_address));
}

#[test]
#[should_panic(expected = "Address already registered")]
fn test_register_address_duplicate_rejection() {
    let env = Env::default();
    let (client, owner, _commitment) = setup_with_owner(&env);
    let address_to_register = Address::generate(&env);

    client.register_address(&owner, &address_to_register);
    client.register_address(&owner, &address_to_register);
}

#[test]
fn test_get_master_returns_none_when_not_set() {
    let env = Env::default();
    let (client, _owner, _commitment) = setup_with_owner(&env);

    let master = client.get_master();
    assert_eq!(master, None);
}

#[test]
fn test_is_address_registered_false_for_unknown() {
    let env = Env::default();
    let (client, _owner, _commitment) = setup_with_owner(&env);
    let unknown_address = Address::generate(&env);

    let is_registered = client.is_address_registered(&unknown_address);
    assert!(!is_registered);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_init_address_manager_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    client.init_address_manager(&owner);
    client.init_address_manager(&owner);
}
