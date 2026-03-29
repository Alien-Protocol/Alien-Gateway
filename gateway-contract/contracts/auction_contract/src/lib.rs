#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

pub mod errors;
pub mod events;
pub mod indexed;
pub mod singleton;
pub mod storage;
pub mod types;

// Ensure event symbols are linked from the main
// contract entrypoint module.
use crate::events::{AUCTION_CLOSED, AUCTION_CREATED, BID_PLACED, BID_REFUNDED, USERNAME_CLAIMED};

/// Ensures event symbol constants are referenced from the crate root so the
/// linker does not strip them when compiling to WASM.
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

    pub fn refund_bid(env: Env, id: u32, bidder: Address) {
        bidder.require_auth();

        // Ensure auction is closed
        let status = storage::auction_get_status(&env, id);
        if status != types::AuctionStatus::Closed {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotClosed);
        }

        // Winner cannot claim a refund via this path
        let highest_bidder = storage::auction_get_highest_bidder(&env, id);
        if highest_bidder
            .as_ref()
            .map(|h| h == &bidder)
            .unwrap_or(false)
        {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotWinner);
        }

        // Guard against double refund
        if storage::auction_is_bid_refunded(&env, id, &bidder) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AlreadyClaimed);
        }

        // Retrieve the outbid amount owed to this bidder
        let amount = storage::auction_get_outbid_amount(&env, id, &bidder);
        if amount <= 0 {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::InvalidState);
        }

        // Transfer asset back to bidder (single transfer)
        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&env.current_contract_address(), &bidder, &amount);

        // Mark refund as complete and zero out the stored amount
        storage::auction_set_bid_refunded(&env, id, &bidder);
        storage::auction_set_outbid_amount(&env, id, &bidder, 0);

        // Emit a single refund event
        events::emit_bid_refunded(
            &env,
            &BytesN::from_array(&env, &[0u8; 32]),
            &bidder,
            amount,
        );
    }

    pub fn close_auction_by_id(env: Env, id: u32) {
        indexed::close_auction_by_id(&env, id)
    }

    pub fn claim(env: Env, id: u32, claimant: Address) {
        indexed::claim(&env, id, claimant)
    }

    pub fn get_auction_info(
        env: Env,
        id: u32,
    ) -> Option<(
        Address,
        Address,
        i128,
        u64,
        i128,
        Option<Address>,
        types::AuctionStatus,
        bool,
    )> {
        if !storage::auction_exists(&env, id) {
            return None;
        }
        Some((
            storage::auction_get_seller(&env, id),
            storage::auction_get_asset(&env, id),
            storage::auction_get_min_bid(&env, id),
            storage::auction_get_end_time(&env, id),
            storage::auction_get_highest_bid(&env, id),
            storage::auction_get_highest_bidder(&env, id),
            storage::auction_get_status(&env, id),
            storage::auction_is_claimed(&env, id),
        ))
    }
}