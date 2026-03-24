#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    Address, BytesN, Env, IntoVal,
};

// Dummy factory contract
#[contract]
pub struct DummyFactory;
#[contractimpl]
impl DummyFactory {
    pub fn deploy_username(env: Env, username_hash: BytesN<32>, claimer: Address) {
        env.events()
            .publish((symbol_short!("deploy"), username_hash), claimer);
    }
}

#[test]
fn test_claim_username_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let factory_id = env.register_contract(None, DummyFactory);
    let claimer = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);

    env.as_contract(&contract_id, || {
        storage::set_factory_contract(&env, &factory_id);
        storage::set_highest_bidder(&env, &claimer);
        storage::set_status(&env, types::AuctionStatus::Closed);
    });

    client.claim_username(&username_hash, &claimer);

    let events = env.events().all();
    assert!(events.len() > 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_not_winner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let factory_id = env.register_contract(None, DummyFactory);
    let winner = Address::generate(&env);
    let not_winner = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);

    env.as_contract(&contract_id, || {
        storage::set_factory_contract(&env, &factory_id);
        storage::set_highest_bidder(&env, &winner);
        storage::set_status(&env, types::AuctionStatus::Closed);
    });
    client.claim_username(&username_hash, &not_winner);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_already_claimed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let factory_id = env.register_contract(None, DummyFactory);
    let claimer = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);

    env.as_contract(&contract_id, || {
        storage::set_factory_contract(&env, &factory_id);
        storage::set_highest_bidder(&env, &claimer);
        storage::set_status(&env, types::AuctionStatus::Claimed);
    });
    client.claim_username(&username_hash, &claimer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_not_closed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let factory_id = env.register_contract(None, DummyFactory);
    let claimer = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);

    env.as_contract(&contract_id, || {
        storage::set_factory_contract(&env, &factory_id);
        storage::set_highest_bidder(&env, &claimer);
        storage::set_status(&env, types::AuctionStatus::Open);
    });
    client.claim_username(&username_hash, &claimer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_no_factory_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let claimer = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);

    env.as_contract(&contract_id, || {
        storage::set_highest_bidder(&env, &claimer);
        storage::set_status(&env, types::AuctionStatus::Closed);
    });
    client.claim_username(&username_hash, &claimer);
}
