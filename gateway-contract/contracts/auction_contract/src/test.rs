#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Env};

fn setup_test(env: &Env) -> (AuctionContractClient<'static>, Address, Address) {
    let contract_id = env.register(AuctionContract, ());
    let client = AuctionContractClient::new(env, &contract_id);

    let seller = Address::generate(env);
    let token_admin = Address::generate(env);
    let asset_id_obj = env.register_stellar_asset_contract_v2(token_admin.clone());
    let asset_id = asset_id_obj.address();

    (client, seller, asset_id)
}

#[test]
fn test_auction_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    let asset_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset_id);
    let asset = soroban_sdk::token::Client::new(&env, &asset_id);

    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    asset_admin.mint(&bidder1, &1000);
    asset_admin.mint(&bidder2, &1000);

    let end_time = 1000;
    client.create_auction(&1, &seller, &asset_id, &100, &end_time);

    client.place_bid(&1, &bidder1, &150);
    client.place_bid(&1, &bidder2, &200);

    assert_eq!(asset.balance(&bidder1), 1000);
    assert_eq!(asset.balance(&client.address), 200);

    env.ledger().set_timestamp(end_time + 1);
    client.close_auction(&1);
    client.claim(&1, &bidder2);
}

#[test]
fn test_auction_no_bids_close() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    let end_time = 1000;
    client.create_auction(&1, &seller, &asset_id, &100, &end_time);
    env.ledger().set_timestamp(end_time + 1);
    client.close_auction(&1);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_place_bid_too_low_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    let bidder = Address::generate(&env);
    client.place_bid(&1, &bidder, &50);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_place_bid_after_close_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    env.ledger().set_timestamp(1001);
    let bidder = Address::generate(&env);
    client.place_bid(&1, &bidder, &150);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_close_auction_early_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    env.ledger().set_timestamp(500);
    client.close_auction(&1);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_claim_not_winner_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    let asset_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset_id);
    let bidder = Address::generate(&env);
    let loser = Address::generate(&env);
    asset_admin.mint(&bidder, &200);

    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    client.place_bid(&1, &bidder, &150);
    env.ledger().set_timestamp(1001);
    client.close_auction(&1);
    client.claim(&1, &loser);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_create_duplicate_auction_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    client.create_auction(&1, &seller, &asset_id, &200, &2000);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_claim_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset_id) = setup_test(&env);
    let asset_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset_id);
    let bidder = Address::generate(&env);
    asset_admin.mint(&bidder, &200);

    client.create_auction(&1, &seller, &asset_id, &100, &1000);
    client.place_bid(&1, &bidder, &150);
    env.ledger().set_timestamp(1001);
    client.close_auction(&1);

    client.claim(&1, &bidder);
    client.claim(&1, &bidder);
}
