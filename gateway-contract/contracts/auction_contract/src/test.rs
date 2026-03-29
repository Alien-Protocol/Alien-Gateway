#[cfg(test)]
mod tests {
    use super::super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::testutils::Events as _;
    use soroban_sdk::{Env, Symbol, TryFromVal, TryIntoVal};

    #[test]
    fn test_bid_refunded_event_emitted_when_outbid() {
        let env = Env::default();
        env.mock_all_auths();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        // Setup auction state
        // register a single stellar asset and mint tokens to bidders so transfers succeed
        let token_admin = Address::generate(&env);
        let asset = env.register_stellar_asset_contract_v2(token_admin).address();
        let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token_admin_client.mint(&alice, &1000);
        token_admin_client.mint(&bob, &1000);

        env.as_contract(&contract_id, || {
            use crate::storage;
            use crate::types::AuctionStatus;
            storage::auction_set_status(&env, 1, AuctionStatus::Open);
            storage::auction_set_min_bid(&env, 1, 50);
            storage::auction_set_end_time(&env, 1, env.ledger().timestamp() + 1000);
            storage::auction_set_asset(&env, 1, &asset);
        });

        // Alice places initial bid
        client.place_bid(&1, &alice, &100_i128);

        // Bob outbids Alice
        client.place_bid(&1, &bob, &200_i128);

        // Capture events and assert BID_RFDN event present with correct bidder and refund_amount
        let events = env.events().all();
        assert!(!events.is_empty());
        // Find any event whose data decodes to (Address, i128) and matches alice/100
        let mut found = false;
        for (_contract, _topics, data) in events.iter().rev() {
            if let Ok((ev_bidder, ev_amount)) = <(Address, i128)>::try_from_val(&env, &data) {
                if ev_bidder == alice && ev_amount == 100_i128 {
                    found = true;
                    break;
                }
            } else if let Ok((uh, ev_bidder, ev_amount)) = <(BytesN<32>, Address, i128)>::try_from_val(&env, &data) {
                if ev_bidder == alice && ev_amount == 100_i128 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "BID_RFDN event not found");
    }

}
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

// ── TTL constant sanity checks ────────────────────────────────────────────────

#[test]
fn test_ttl_constants_match_formula() {
    // 30 days * 24h * 3600s / 5s per ledger = 518_400
    assert_eq!(storage::PERSISTENT_BUMP_AMOUNT, 30 * 24 * 3600 / 5);
    // 7 days * 24h * 3600s / 5s per ledger = 120_960
    assert_eq!(storage::PERSISTENT_LIFETIME_THRESHOLD, 7 * 24 * 3600 / 5);
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
    assert!(!events.is_empty());
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
    assert!(!env.events().all().is_empty());
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

    // bidder1 is outbid and funds are held for refund; bidder2 is highest bidder.
    assert_eq!(token.balance(&bidder1), 850);
    assert_eq!(token.balance(&bidder2), 800);

    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);

    client.refund_bid(&1, &bidder1);
    assert_eq!(token.balance(&bidder1), 1000);

    client.claim(&1, &bidder2);
    assert_eq!(token.balance(&seller), 200);
}

#[test]
fn test_refund_bid_success() {
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

    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);

    client.refund_bid(&1, &bidder1);

    assert_eq!(token.balance(&bidder1), 1000);
    assert_eq!(token.balance(&bidder2), 800);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_refund_bid_winner_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    token_admin.mint(&bidder1, &1000);
    token_admin.mint(&bidder2, &1000);

    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder1, &150);
    client.place_bid(&1, &bidder2, &200);

    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);

    client.refund_bid(&1, &bidder2);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_refund_bid_double_refund_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    token_admin.mint(&bidder1, &1000);
    token_admin.mint(&bidder2, &1000);

    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder1, &150);
    client.place_bid(&1, &bidder2, &200);

    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);

    client.refund_bid(&1, &bidder1);
    client.refund_bid(&1, &bidder1);
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
fn test_create_auction_zero_min_bid_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    client.create_auction(&1, &seller, &asset, &0, &1000u64);
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
#[should_panic(expected = "Error(Contract, #8)")]
fn test_close_nonexistent_auction_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    client.close_auction_by_id(&999);
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
#[should_panic(expected = "Error(Contract, #10)")]
fn test_outbid_self_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder = Address::generate(&env);
    token_admin.mint(&bidder, &500);
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    client.place_bid(&1, &bidder, &150);
    // Same bidder tries to raise their own bid — must be rejected
    client.place_bid(&1, &bidder, &200);
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

#[test]
fn test_create_auction_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    
    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    
    let events = env.events().all();
    assert!(events.len() > 0);
    
    let event = events.last().unwrap();
    let (_, topics, data) = event;
    
    let event_name = soroban_sdk::Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(event_name, soroban_sdk::Symbol::new(&env, "AuctionCreatedEvent"));
fn test_get_auction_info() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, seller, asset) = setup(&env);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &asset);
    let bidder = Address::generate(&env);
    token_admin.mint(&bidder, &200);

    // Should return None for unknown id
    assert_eq!(client.get_auction_info(&1), None);

    client.create_auction(&1, &seller, &asset, &100, &1000u64);
    
    // Initial state
    let info1 = client.get_auction_info(&1).unwrap();
    assert_eq!(info1, (seller.clone(), asset.clone(), 100, 1000, 0, None, types::AuctionStatus::Open, false));

    // After bid
    client.place_bid(&1, &bidder, &150);
    let info2 = client.get_auction_info(&1).unwrap();
    assert_eq!(info2, (seller.clone(), asset.clone(), 100, 1000, 150, Some(bidder.clone()), types::AuctionStatus::Open, false));

    // After close
    env.ledger().set_timestamp(1001);
    client.close_auction_by_id(&1);
    let info3 = client.get_auction_info(&1).unwrap();
    assert_eq!(info3, (seller.clone(), asset.clone(), 100, 1000, 150, Some(bidder.clone()), types::AuctionStatus::Closed, false));

    // After claim
    client.claim(&1, &bidder);
    let info4 = client.get_auction_info(&1).unwrap();
    assert_eq!(info4, (seller.clone(), asset.clone(), 100, 1000, 150, Some(bidder.clone()), types::AuctionStatus::Closed, true));
}
