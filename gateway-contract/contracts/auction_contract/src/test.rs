#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, BytesN, Env,
};

// Dummy factory contract (kept for existing tests)
#[contract]
pub struct DummyFactory;
#[contractimpl]
impl DummyFactory {
    pub fn deploy_username(_env: Env, _username_hash: BytesN<32>, _claimer: Address) {}
}

// ── existing tests ────────────────────────────────────────────────────────────

#[test]
fn test_claim_username_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let factory_id = env.register(DummyFactory, ());
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
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let factory_id = env.register(DummyFactory, ());
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
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let factory_id = env.register(DummyFactory, ());
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
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let factory_id = env.register(DummyFactory, ());
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
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let claimer = Address::generate(&env);
    let username_hash = BytesN::from_array(&env, &[0; 32]);
    env.as_contract(&contract_id, || {
        storage::set_highest_bidder(&env, &claimer);
        storage::set_status(&env, types::AuctionStatus::Closed);
    });
    client.claim_username(&username_hash, &claimer);
}

#[test]
fn test_close_auction_success() {
    let env = Env::default();
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let username_hash = BytesN::from_array(&env, &[1; 32]);
    let bidder = Address::generate(&env);
    env.as_contract(&contract_id, || {
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 1000);
        storage::set_highest_bidder(&env, &bidder);
        storage::set_highest_bid(&env, 100);
    });
    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });
    client.close_auction(&username_hash);
    env.as_contract(&contract_id, || {
        assert_eq!(storage::get_status(&env), types::AuctionStatus::Closed);
    });
}

#[test]
fn test_close_auction_zero_bid() {
    let env = Env::default();
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let username_hash = BytesN::from_array(&env, &[2; 32]);
    env.as_contract(&contract_id, || {
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 1000);
        storage::set_highest_bid(&env, 0);
    });
    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });
    client.close_auction(&username_hash);
    env.as_contract(&contract_id, || {
        assert_eq!(storage::get_status(&env), types::AuctionStatus::Closed);
    });
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_close_auction_not_expired() {
    let env = Env::default();
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let username_hash = BytesN::from_array(&env, &[3; 32]);
    let bidder = Address::generate(&env);
    env.as_contract(&contract_id, || {
        storage::set_status(&env, types::AuctionStatus::Open);
        storage::set_end_time(&env, 5000);
        storage::set_highest_bidder(&env, &bidder);
        storage::set_highest_bid(&env, 100);
    });
    env.ledger().with_mut(|l| {
        l.timestamp = 2000;
    });
    client.close_auction(&username_hash);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_close_auction_not_open() {
    let env = Env::default();
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(&env, &contract_id);
    let username_hash = BytesN::from_array(&env, &[4; 32]);
    env.as_contract(&contract_id, || {
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
    let contract_id = env.register(AuctionContract, ());
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
    assert!(env.events().all().len() > 0);
}

// ── new lifecycle tests (issue #101) ─────────────────────────────────────────

fn setup(env: &Env) -> (AuctionContractClient<'static>, Address, Address) {
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(env, &contract_id);
    let seller = Address::generate(env);
    let token_admin = Address::generate(env);
    let asset = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    (client, seller, asset)
}

#[test]
fn test_auction_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let token = soroban_sdk::token::Client::new(&env, &asset);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    token_admin.mint(&bidder1, &1000);
    token_admin.mint(&bidder2, &1000);

    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder1, &150);
    client.place_bid(&1, &bidder2, &200);

    // bidder1 refunded, bidder2 holds 800
    assert_eq!(token.balance(&bidder1), 1000);
    assert_eq!(token.balance(&bidder2), 800);

    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);
    client.claim(&1, &bidder2);

    assert_eq!(token.balance(&seller), 200);
}

#[test]
fn test_auction_no_bids_close() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_place_bid_too_low_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    let bidder = Address::generate(&env);
    client.place_bid(&1, &bidder, &50);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_place_bid_after_close_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    env.ledger().set_timestamp(1001);
    let bidder = Address::generate(&env);
    client.place_bid(&1, &bidder, &150);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_close_auction_early_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    env.ledger().set_timestamp(500);
    client.close_auction_by_id(&1);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_claim_not_winner_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder = Address::generate(&env);
    let loser = Address::generate(&env);
    token_admin.mint(&bidder, &200);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder, &150);
    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);
    client.claim(&1, &loser);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_auction_past_end_time_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    env.ledger().set_timestamp(2000);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_create_duplicate_auction_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.create_auction(&1, &seller, &asset, &200, &2000u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_claim_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder = Address::generate(&env);
    token_admin.mint(&bidder, &200);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder, &150);
    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);
    client.claim(&1, &bidder);
    client.claim(&1, &bidder);
}
