use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};
use alien_gateway::CoreContract;

#[test]
fn test_successful_init() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let user = Symbol::new(&env, "alien_user");

    CoreContract::init(env.clone(), user.clone(), owner.clone());

    assert_eq!(CoreContract::get_username(env.clone()), user);
    assert_eq!(CoreContract::get_owner(env.clone()), owner);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_double_init_panics() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let user = Symbol::new(&env, "alien_user");

    CoreContract::init(env.clone(), user.clone(), owner.clone());
    CoreContract::init(env.clone(), user.clone(), owner);
}

#[test]
#[should_panic(expected = "Username cannot be empty")]
fn test_invalid_username_empty() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let user = Symbol::new(&env, "");

    CoreContract::init(env.clone(), user, owner);
}

#[test]
#[should_panic]
fn test_invalid_username_long() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let user = Symbol::new(&env, "this_username_is_way_more_than_32_characters");

    CoreContract::init(env.clone(), user, owner);
}
