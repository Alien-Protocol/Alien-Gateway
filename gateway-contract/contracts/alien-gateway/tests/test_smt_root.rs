use alien_gateway::{AddressManager, Contract, SmtRoot};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, BytesN, Env,
};

fn make_root(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

#[test]
fn test_get_root_before_set_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });

    let root = env.as_contract(&contract_id, || SmtRoot::get_root(env.clone()));
    assert!(root.is_none());
}

#[test]
fn test_set_root_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let root_val = make_root(&env, 0xab);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), root_val.clone());
    });

    let stored = env
        .as_contract(&contract_id, || SmtRoot::get_root(env.clone()))
        .unwrap();
    assert_eq!(stored, root_val);
}

#[test]
fn test_update_root_replaces_previous() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let root1 = make_root(&env, 0x11);
    let root2 = make_root(&env, 0x22);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), root1.clone());
    });
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), root2.clone());
    });

    let stored = env
        .as_contract(&contract_id, || SmtRoot::get_root(env.clone()))
        .unwrap();
    assert_eq!(stored, root2);
}

#[test]
fn test_update_root_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let root_val = make_root(&env, 0xaa);

    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), root_val.clone());
    });

    // Verify one ROOT_UPD event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected 1 ROOT_UPD event");
}

#[test]
#[should_panic]
fn test_update_root_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let root_val = make_root(&env, 0xff);

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        AddressManager::init(env.clone(), owner.clone());
    });

    // Remove all auth — next call should panic
    env.set_auths(&[]);
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), root_val.clone());
    });
}
