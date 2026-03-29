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
        bidder.require_auth();
        let status = storage::auction_get_status(&env, id);
        if status != types::AuctionStatus::Open {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotOpen);
        }

        let end_time = storage::auction_get_end_time(&env, id);
        if env.ledger().timestamp() >= end_time {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotOpen);
        }

        let min_bid = storage::auction_get_min_bid(&env, id);
        let highest_bid = storage::auction_get_highest_bid(&env, id);
        if amount < min_bid || amount <= highest_bid {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::BidTooLow);
        }

        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);

        // Accept bid funds into contract.
        token.transfer(&bidder, env.current_contract_address(), &amount);

        if let Some(prev_bidder) = storage::auction_get_highest_bidder(&env, id) {
            // Record outbid amount for later refund by the bidder.
            let prev_amount = highest_bid;
            let existing_outbid = storage::auction_get_outbid_amount(&env, id, &prev_bidder);
            storage::auction_set_outbid_amount(
                &env,
                id,
                &prev_bidder,
                existing_outbid + prev_amount,
            );
        }

        storage::auction_set_highest_bidder(&env, id, &bidder);
        storage::auction_set_highest_bid(&env, id, amount);
    }

    pub fn refund_bid(env: Env, id: u32, bidder: Address) {
        bidder.require_auth();

        let status = storage::auction_get_status(&env, id);
        if status != types::AuctionStatus::Closed {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotClosed);
        }

        let highest_bidder = storage::auction_get_highest_bidder(&env, id);
        if highest_bidder
            .as_ref()
            .map(|h| h == &bidder)
            .unwrap_or(false)
        {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotWinner);
        }

        if storage::auction_is_bid_refunded(&env, id, &bidder) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AlreadyClaimed);
        }

        let refund_amount = storage::auction_get_outbid_amount(&env, id, &bidder);
        if refund_amount <= 0 {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::InvalidState);
        }

        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);

        storage::auction_set_bid_refunded(&env, id, &bidder);
        storage::auction_set_outbid_amount(&env, id, &bidder, 0);

        token.transfer(&env.current_contract_address(), &bidder, &refund_amount);
        events::emit_bid_refunded(
            &env,
            &BytesN::from_array(&env, &[0u8; 32]),
            &bidder,
            refund_amount,
        );
    }

    pub fn close_auction_by_id(env: Env, id: u32) {
        indexed::close_auction_by_id(&env, id)
    }

    pub fn claim(env: Env, id: u32, claimant: Address) {
        indexed::claim(&env, id, claimant)
    }
}
