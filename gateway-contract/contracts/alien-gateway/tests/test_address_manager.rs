use alien_gateway::{AddressManager, Contract};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

#[test]
fn test_master_assignment() {
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
fn test_switch_master() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user1.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::register_address(env.clone(), user2.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user1.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user2.clone());
    });

    let master = env
        .as_contract(&contract_id, || AddressManager::get_master(env.clone()))
        .unwrap();
    assert_eq!(master, user2);
}

#[test]
#[should_panic(expected = "Address does not exist")]
fn test_non_existent_address_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });
}

#[test]
#[should_panic]
fn test_unauthorized_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let attacker = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });

    env.as_contract(&contract_id, || {
        owner.require_auth();
        AddressManager::register_address(env.clone(), user.clone());
    });

    env.as_contract(&contract_id, || {
        attacker.require_auth();
        AddressManager::set_master_stellar_address(env.clone(), user.clone());
    });
}

// --- add_stellar_address tests ---

#[test]
fn test_add_stellar_address_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let label = symbol_short!("wallet1");

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), user.clone(), label.clone());
    });

    let metadata = env
        .as_contract(&contract_id, || {
            AddressManager::get_stellar_address(env.clone(), user.clone())
        })
        .unwrap();
    assert_eq!(metadata.label, label);
}

#[test]
#[should_panic(expected = "Address already exists")]
fn test_add_stellar_address_duplicate_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let label = symbol_short!("wallet1");

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), user.clone(), label.clone());
    });
    // Second add of same address should panic
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), user.clone(), label.clone());
    });
}

#[test]
#[should_panic]
fn test_add_stellar_address_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let label = symbol_short!("hack");

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });

    // No mock_all_auths for attacker — auth will fail
    env.set_auths(&[]);
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), attacker.clone(), label.clone());
    });
}

#[test]
fn test_add_stellar_address_metadata_stored() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let label1 = symbol_short!("primary");
    let label2 = symbol_short!("secondary");

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), user1.clone(), label1.clone());
    });
    env.as_contract(&contract_id, || {
        AddressManager::add_stellar_address(env.clone(), user2.clone(), label2.clone());
    });

    let meta1 = env
        .as_contract(&contract_id, || {
            AddressManager::get_stellar_address(env.clone(), user1.clone())
        })
        .unwrap();
    let meta2 = env
        .as_contract(&contract_id, || {
            AddressManager::get_stellar_address(env.clone(), user2.clone())
        })
        .unwrap();

    assert_eq!(meta1.label, label1);
    assert_eq!(meta2.label, label2);
}
