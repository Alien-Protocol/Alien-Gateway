#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

pub mod errors;
pub mod events;
pub mod indexed;
pub mod singleton;
pub mod storage;
pub mod types;

// Ensure event symbols are linked from the main contract entrypoint module.
use crate::events::{AUCTION_CLOSED, AUCTION_CREATED, BID_PLACED, BID_REFUNDED, USERNAME_CLAIMED};

#[allow(dead_code)]
fn _touch_event_symbols() {
    let _ = (
        AUCTION_CREATED,
        BID_PLACED,
        AUCTION_CLOSED,
        USERNAME_CLAIMED,
        BID_REFUNDED,
    );
}

#[cfg(test)]
mod test;

#[contract]
pub struct AuctionContract;

/// Singleton flow: one auction per contract instance.
#[contractimpl]
impl AuctionContract {
    pub fn close_auction(
        env: Env,
        username_hash: BytesN<32>,
    ) -> Result<(), errors::AuctionError> {
        singleton::close_auction(&env, username_hash)
    }

    pub fn claim_username(
        env: Env,
        username_hash: BytesN<32>,
        claimer: Address,
    ) -> Result<(), errors::AuctionError> {
        singleton::claim_username(&env, username_hash, claimer)
    }
}

/// ID-indexed flow: multiple auctions identified by a numeric id.
#[contractimpl]
impl AuctionContract {
    pub fn create_auction(
        env: Env,
        id: u32,
        seller: Address,
        asset: Address,
        min_bid: i128,
        end_time: u64,
    ) {
        indexed::create_auction(&env, id, seller, asset, min_bid, end_time)
    }

    pub fn place_bid(env: Env, id: u32, bidder: Address, amount: i128) {
        indexed::place_bid(&env, id, bidder, amount)
    }

    pub fn close_auction_by_id(env: Env, id: u32) {
        indexed::close_auction_by_id(&env, id)
    }

    pub fn claim(env: Env, id: u32, claimant: Address) {
        indexed::claim(&env, id, claimant)
    }
}
