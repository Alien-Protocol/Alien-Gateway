/// ID-indexed auction flow: multiple auctions identified by a numeric `id`.
/// Handles `create_auction`, `place_bid`, `close_auction_by_id`, and `claim`.
use soroban_sdk::{Address, Env};

use crate::{errors, storage, types};

pub fn create_auction(
    env: &Env,
    id: u32,
    seller: Address,
    asset: Address,
    min_bid: i128,
    end_time: u64,
) {
    seller.require_auth();
    if storage::auction_exists(env, id) {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::AuctionNotOpen);
    }
    if end_time <= env.ledger().timestamp() {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::AuctionNotClosed);
    }
    storage::auction_set_seller(env, id, &seller);
    storage::auction_set_asset(env, id, &asset);
    storage::auction_set_min_bid(env, id, min_bid);
    storage::auction_set_end_time(env, id, end_time);
    storage::auction_set_status(env, id, types::AuctionStatus::Open);
}

pub fn place_bid(env: &Env, id: u32, bidder: Address, amount: i128) {
    bidder.require_auth();
    let end_time = storage::auction_get_end_time(env, id);
    if env.ledger().timestamp() >= end_time {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::AuctionNotOpen);
    }
    let min_bid = storage::auction_get_min_bid(env, id);
    let highest_bid = storage::auction_get_highest_bid(env, id);
    if amount < min_bid || amount <= highest_bid {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::BidTooLow);
    }
    let asset = storage::auction_get_asset(env, id);
    let token = soroban_sdk::token::Client::new(env, &asset);
    token.transfer(&bidder, env.current_contract_address(), &amount);
    if let Some(prev_bidder) = storage::auction_get_highest_bidder(env, id) {
        token.transfer(&env.current_contract_address(), &prev_bidder, &highest_bid);
    }
    storage::auction_set_highest_bidder(env, id, &bidder);
    storage::auction_set_highest_bid(env, id, amount);
}

pub fn close_auction_by_id(env: &Env, id: u32) {
    let end_time = storage::auction_get_end_time(env, id);
    if env.ledger().timestamp() < end_time {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::AuctionNotClosed);
    }
    storage::auction_set_status(env, id, types::AuctionStatus::Closed);
}

pub fn claim(env: &Env, id: u32, claimant: Address) {
    claimant.require_auth();
    let status = storage::auction_get_status(env, id);
    if status != types::AuctionStatus::Closed {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::NotClosed);
    }
    if storage::auction_is_claimed(env, id) {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::AlreadyClaimed);
    }
    let winner = storage::auction_get_highest_bidder(env, id);
    if winner.as_ref().map(|w| w == &claimant).unwrap_or(false) {
        let asset = storage::auction_get_asset(env, id);
        let token = soroban_sdk::token::Client::new(env, &asset);
        let winning_bid = storage::auction_get_highest_bid(env, id);
        let seller = storage::auction_get_seller(env, id);
        token.transfer(&env.current_contract_address(), &seller, &winning_bid);
        storage::auction_set_claimed(env, id);
    } else {
        soroban_sdk::panic_with_error!(env, errors::AuctionError::NotWinner);
    }
}
