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

// Tests for close_auction

#[test]
fn test_close_auction_success() {
    let env = Env::default();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let username_hash = BytesN::from_array(&env, &[1; 32]);
    let bidder = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Set up auction state
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 1000);
        storage::set_highest_bidder(&env, &bidder);
        storage::set_highest_bid(&env, 100);
    });

    // Advance ledger to make current_time > end_time
    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });

    let result = client.close_auction(&username_hash);
    assert!(result.is_ok());

    // Verify status changed to Closed
    env.as_contract(&contract_id, || {
        let status = storage::get_status(&env);
        assert_eq!(status, types::AuctionStatus::Closed);
    });
}

#[test]
fn test_close_auction_zero_bid() {
    let env = Env::default();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let username_hash = BytesN::from_array(&env, &[2; 32]);

    env.as_contract(&contract_id, || {
        // Set up auction state with no bidder (zero-bid auction)
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 1000);
        storage::set_highest_bid(&env, 0);
    });

    // Advance ledger to make current_time > end_time
    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });

    let result = client.close_auction(&username_hash);
    assert!(result.is_ok());

    // Verify status changed to Closed
    env.as_contract(&contract_id, || {
        let status = storage::get_status(&env);
        assert_eq!(status, types::AuctionStatus::Closed);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_close_auction_not_expired() {
    let env = Env::default();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let username_hash = BytesN::from_array(&env, &[3; 32]);
    let bidder = Address::generate(&env);

    env.as_contract(&contract_id, || {
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 5000); // End time is in the future
        storage::set_highest_bidder(&env, &bidder);
        storage::set_highest_bid(&env, 100);
    });

    env.ledger().with_mut(|l| {
        l.timestamp = 2000; // Current time is before end time
    });

    client.close_auction(&username_hash);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_close_auction_not_open() {
    let env = Env::default();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let username_hash = BytesN::from_array(&env, &[4; 32]);

    env.as_contract(&contract_id, || {
        // Auction is already closed
        storage::set_status(&env, types::AuctionStatus::Closed);
        storage::set_end_time(&env, 1000);
    });

    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });

    client.close_auction(&username_hash);
}

#[test]
fn test_close_auction_emits_event() {
    let env = Env::default();

    let contract_id = env.register_contract(None, AuctionContract);
    let client = AuctionContractClient::new(&env, &contract_id);

    let username_hash = BytesN::from_array(&env, &[5; 32]);
    let bidder = Address::generate(&env);

    env.as_contract(&contract_id, || {
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 1000);
        storage::set_highest_bidder(&env, &bidder);
        storage::set_highest_bid(&env, 500);
    });

    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });

    client.close_auction(&username_hash);

    // Verify event was emitted
    let events = env.events().all();
    assert!(events.len() > 0);
}
