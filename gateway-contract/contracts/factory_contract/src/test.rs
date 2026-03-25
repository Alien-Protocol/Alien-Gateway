#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{contract, contractimpl, IntoVal, Symbol, TryFromVal, Val, Vec};
use soroban_sdk::{Address, BytesN, Env};

use crate::errors::FactoryError;
use crate::events::USERNAME_DEPLOYED;
use crate::{FactoryContract, FactoryContractClient};

#[contract]
struct StubContract;

#[contractimpl]
impl StubContract {}

fn setup_factory(env: &Env) -> (Address, FactoryContractClient<'_>, Address, Address) {
    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(env, &factory_id);
    let auction_contract = env.register(StubContract, ());
    let core_contract = env.register(StubContract, ());

    factory.configure(&auction_contract, &core_contract);

    (factory_id, factory, auction_contract, core_contract)
}

fn username_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[7; 32])
}

// ============================================================================
// OFFICIAL UPSTREAM TESTS
// ============================================================================

#[test]
fn deploy_username_stores_record_and_emits_event() {
    let env = Env::default();
    let (factory_id, factory, auction_contract, core_contract) = setup_factory(&env);
    let owner = Address::generate(&env);
    let hash = username_hash(&env);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);
    factory.deploy_username(&hash, &owner);

    let events = env.events().all();

    let record = factory.get_username_record(&hash).unwrap();
    assert_eq!(record.username_hash, hash);
    assert_eq!(record.owner, owner);
    assert_eq!(record.registered_at, env.ledger().timestamp());
    assert_eq!(record.core_contract, core_contract);
    assert_eq!(events.len(), 1);

    let (event_contract, topics, data) = events.get(0).unwrap();
    assert_eq!(event_contract, factory_id);
    assert_eq!(topics.len(), 1);

    let event_name = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    let (event_hash, event_owner, event_registered_at) =
        <(BytesN<32>, Address, u64)>::try_from_val(&env, &data).unwrap();

    assert_eq!(event_name, USERNAME_DEPLOYED);
    assert_eq!(event_hash, hash);
    assert_eq!(event_owner, owner);
    assert_eq!(event_registered_at, record.registered_at);
}

#[test]
fn duplicate_deployment_is_rejected() {
    let env = Env::default();
    let (factory_id, factory, auction_contract, _) = setup_factory(&env);
    let owner = Address::generate(&env);
    let hash = username_hash(&env);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args.clone(),
            sub_invokes: &[],
        },
    }]);
    factory.deploy_username(&hash, &owner);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);
    let result = env.try_invoke_contract::<(), FactoryError>(
        &factory_id,
        &Symbol::new(&env, "deploy_username"),
        Vec::<Val>::from_array(
            &env,
            [hash.clone().into_val(&env), owner.clone().into_val(&env)],
        ),
    );

    assert_eq!(result, Err(Ok(FactoryError::AlreadyDeployed)));
}

#[test]
fn non_registered_auction_auth_is_rejected() {
    let env = Env::default();
    let (factory_id, _, auction_contract, _) = setup_factory(&env);
    let wrong_caller = env.register(StubContract, ());
    let owner = Address::generate(&env);
    let hash = username_hash(&env);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &wrong_caller,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);
    let result = env.try_invoke_contract::<(), FactoryError>(
        &factory_id,
        &Symbol::new(&env, "deploy_username"),
        Vec::<Val>::from_array(&env, [hash.into_val(&env), owner.into_val(&env)]),
    );

    assert!(result.is_err());
    assert_ne!(wrong_caller, auction_contract);
}

#[test]
fn get_username_owner_returns_owner_after_deploy() {
    let env = Env::default();
    let (factory_id, factory, auction_contract, _) = setup_factory(&env);
    let owner = Address::generate(&env);
    let hash = username_hash(&env);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);
    factory.deploy_username(&hash, &owner);

    assert_eq!(factory.get_username_owner(&hash), Some(owner));
}

#[test]
fn get_username_owner_returns_none_for_unregistered_hash() {
    let env = Env::default();
    let (_, factory, _, _) = setup_factory(&env);
    let unknown_hash = BytesN::from_array(&env, &[0xFF; 32]);

    assert_eq!(factory.get_username_owner(&unknown_hash), None);
}

// ============================================================================
// ISSUE #108 SUPPLEMENTARY TESTS
// ============================================================================

#[test]
fn test_deploy_username_success() {
    let env = Env::default();
    let (factory_id, factory, auction_contract, _core_contract) = setup_factory(&env);
    let owner = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[10; 32]);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);

    factory.deploy_username(&hash, &owner);
    let record = factory.get_username_record(&hash).unwrap();
    assert_eq!(record.owner, owner);
}

#[test]
fn test_deploy_username_duplicate_fails() {
    let env = Env::default();
    let (factory_id, factory, auction_contract, _) = setup_factory(&env);
    let owner = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[11; 32]);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args.clone(),
            sub_invokes: &[],
        },
    }]);
    factory.deploy_username(&hash, &owner);

    env.mock_auths(&[MockAuth {
        address: &auction_contract,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);

    let result = env.try_invoke_contract::<(), FactoryError>(
        &factory_id,
        &Symbol::new(&env, "deploy_username"),
        Vec::<Val>::from_array(
            &env,
            [hash.clone().into_val(&env), owner.clone().into_val(&env)],
        ),
    );

    assert_eq!(result, Err(Ok(FactoryError::AlreadyDeployed)));
}

#[test]
fn test_deploy_unauthorized_fails() {
    let env = Env::default();
    let (factory_id, _, _, _) = setup_factory(&env);
    let wrong_caller = Address::generate(&env);
    let owner = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[12; 32]);
    let deploy_args: Vec<Val> = (hash.clone(), owner.clone()).into_val(&env);

    env.mock_auths(&[MockAuth {
        address: &wrong_caller,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "deploy_username",
            args: deploy_args,
            sub_invokes: &[],
        },
    }]);

    let result = env.try_invoke_contract::<(), FactoryError>(
        &factory_id,
        &Symbol::new(&env, "deploy_username"),
        Vec::<Val>::from_array(
            &env,
            [hash.clone().into_val(&env), owner.clone().into_val(&env)],
        ),
    );

    assert!(result.is_err());
}

#[test]
fn test_get_owner_none_for_unknown() {
    let env = Env::default();
    let (_, factory, _, _) = setup_factory(&env);
    let unknown_hash = BytesN::from_array(&env, &[99; 32]);
    let record = factory.get_username_record(&unknown_hash);
    assert!(record.is_none());
}
