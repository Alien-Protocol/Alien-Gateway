//! Tests for CoreContract::transfer — username commitment ownership transfer.

use alien_gateway::{Contract, CoreContract, SmtRoot};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, BytesN, Env, Symbol,
};

fn make_root(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn setup(env: &Env) -> (soroban_sdk::Address, Address, Symbol) {
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(env);
    let username = Symbol::new(env, "alien_user");

    env.as_contract(&contract_id, || {
        CoreContract::init(env.clone(), username.clone(), owner.clone());
    });

    (contract_id, owner, username)
}

// ── Authorized transfer ───────────────────────────────────────────────────────

#[test]
fn test_transfer_updates_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _owner, _username) = setup(&env);
    let new_owner = Address::generate(&env);
    let new_root = make_root(&env, 0xbb);

    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), new_owner.clone(), new_root.clone());
    });

    let stored_owner = env.as_contract(&contract_id, || CoreContract::get_owner(env.clone()));
    assert_eq!(stored_owner, new_owner);
}

#[test]
fn test_transfer_updates_smt_root() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _owner, _username) = setup(&env);
    let new_owner = Address::generate(&env);
    let new_root = make_root(&env, 0xcc);

    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), new_owner.clone(), new_root.clone());
    });

    let stored_root = env
        .as_contract(&contract_id, || SmtRoot::get_root(env.clone()))
        .unwrap();
    assert_eq!(stored_root, new_root);
}

#[test]
fn test_transfer_emits_transfer_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, owner, username) = setup(&env);
    let new_owner = Address::generate(&env);
    let new_root = make_root(&env, 0xdd);

    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), new_owner.clone(), new_root.clone());
    });

    // Expect 2 events emitted during the transfer call: TRANSFER + ROOT_UPD
    // (INIT from setup runs in a prior as_contract scope and is not captured here)
    let events = env.events().all();
    assert_eq!(events.len(), 2, "Expected TRANSFER and ROOT_UPD events");

    // Verify the TRANSFER event payload (index 0)
    let (_, _topics, data) = events.get(0).unwrap();
    let (ev_username, ev_old_owner, ev_new_owner): (Symbol, Address, Address) =
        soroban_sdk::FromVal::from_val(&env, &data);
    assert_eq!(ev_username, username);
    assert_eq!(ev_old_owner, owner);
    assert_eq!(ev_new_owner, new_owner);
}

#[test]
fn test_transfer_new_owner_can_update_root() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, _owner, _username) = setup(&env);
    let new_owner = Address::generate(&env);
    let new_root = make_root(&env, 0xee);
    let updated_root = make_root(&env, 0xff);

    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), new_owner.clone(), new_root.clone());
    });

    // New owner should now be able to update the SMT root
    env.as_contract(&contract_id, || {
        SmtRoot::update_root(env.clone(), updated_root.clone());
    });

    let stored_root = env
        .as_contract(&contract_id, || SmtRoot::get_root(env.clone()))
        .unwrap();
    assert_eq!(stored_root, updated_root);
}

// ── Unauthorized transfer ─────────────────────────────────────────────────────

#[test]
#[should_panic]
fn test_transfer_unauthorized_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let owner = Address::generate(&env);
    let username = Symbol::new(&env, "alien_user");

    env.mock_all_auths();
    env.as_contract(&contract_id, || {
        CoreContract::init(env.clone(), username.clone(), owner.clone());
    });

    let new_owner = Address::generate(&env);
    let new_root = make_root(&env, 0x01);

    // Strip all auth — transfer must panic
    env.set_auths(&[]);
    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), new_owner.clone(), new_root.clone());
    });
}

#[test]
#[should_panic(expected = "New owner must differ from current owner")]
fn test_transfer_to_same_owner_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, owner, _username) = setup(&env);
    let new_root = make_root(&env, 0x02);

    env.as_contract(&contract_id, || {
        CoreContract::transfer(env.clone(), owner.clone(), new_root.clone());
    });
}
