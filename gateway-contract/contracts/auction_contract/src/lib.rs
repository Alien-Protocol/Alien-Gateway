#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol};
pub mod errors;
pub mod events;
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

#[contractimpl]
impl AuctionContract {
    pub fn close_auction(
        env: Env,
        username_hash: BytesN<32>,
    ) -> Result<(), crate::errors::AuctionError>
    {
        let status = storage::get_status(&env);

        // Reject if status is not Open
        if status != types::AuctionStatus::Open {
            return Err(crate::errors::AuctionError::AuctionNotOpen);
        }

        // Get current ledger timestamp and end time
        let current_time = env.ledger().timestamp();
        let end_time = storage::get_end_time(&env);
        // Reject if timestamp < end_time
        if current_time < end_time {
            return Err(crate::errors::AuctionError::AuctionNotClosed);
        }

        // Set status to Closed
        storage::set_status(&env, types::AuctionStatus::Closed);
        // Get winner and winning bid
        let winner = storage::get_highest_bidder(&env);
        let winning_bid = storage::get_highest_bid(&env);
        // Emit AUCTION_CLOSED event with winner and winning bid
        events::emit_auction_closed(&env, &username_hash, winner.clone(), winning_bid);
        Ok(())
    }

    pub fn claim_username(
        env: Env,
        username_hash: BytesN<32>,
        claimer: Address,
    ) -> Result<(), crate::errors::AuctionError>
    {
        claimer.require_auth();

        let status = storage::get_status(&env);

        if status == types::AuctionStatus::Claimed {
            return Err(crate::errors::AuctionError::AlreadyClaimed);
        }

        if status != types::AuctionStatus::Closed {
            return Err(crate::errors::AuctionError::NotClosed);
        }

        let highest_bidder = storage::get_highest_bidder(&env);
        if !highest_bidder.map(|h| h == claimer).unwrap_or(false) {
            return Err(crate::errors::AuctionError::NotWinner);
        }

        // Set status to Claimed
        storage::set_status(&env, types::AuctionStatus::Claimed);
        // Call factory_contract.deploy_username(username_hash, claimer)
        let factory = storage::get_factory_contract(&env);
        if factory.is_none() {
            return Err(crate::errors::AuctionError::NoFactoryContract);
        }

        let factory_addr = factory.ok_or(crate::errors::AuctionError::NoFactoryContract)?;
        env.invoke_contract::<()>(
            &factory_addr,
            &Symbol::new(&env, "deploy_username"),
            vec![&env, username_hash.into_val(&env), claimer.into_val(&env)],
        );

        // Emit USERNAME_CLAIMED event
        events::emit_username_claimed(&env, &username_hash, &claimer);
        Ok(())
    }
}

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
        seller.require_auth();
        if storage::auction_exists(&env, id) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotOpen);
        }
        if min_bid <= 0 {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::BidTooLow);
        }
        storage::auction_set_seller(&env, id, &seller);
        storage::auction_set_asset(&env, id, &asset);
        storage::auction_set_min_bid(&env, id, min_bid);
        storage::auction_set_end_time(&env, id, end_time);
        storage::auction_set_status(&env, id, types::AuctionStatus::Open);
    }

    pub fn place_bid(env: Env, id: u32, bidder: Address, amount: i128) {
        bidder.require_auth();
        let end_time = storage::auction_get_end_time(&env, id);
        if env.ledger().timestamp() >= end_time {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotOpen);
        }
        let min_bid = storage::auction_get_min_bid(&env, id);
        let highest_bid = storage::auction_get_highest_bid(&env, id);
        if amount < min_bid || amount <= highest_bid {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::BidTooLow);
        }
        if storage::auction_get_highest_bidder(&env, id)
            .map(|h| h == bidder)
            .unwrap_or(false)
        {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::SelfBid);
        }
        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);
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
        // Ensure auction is closed
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
        // Check double refund
        if storage::auction_is_bid_refunded(&env, id, &bidder) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AlreadyClaimed);
        }
        // Get outbid amount
        let amount = storage::auction_get_outbid_amount(&env, id, &bidder);
        if amount <= 0 {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::InvalidState);
        }
        // Transfer asset back to bidder
        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&env.current_contract_address(), &bidder, &amount);
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
        if !storage::auction_exists(&env, id) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotOpen);
        }
        let end_time = storage::auction_get_end_time(&env, id);
        if env.ledger().timestamp() < end_time {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotClosed);
        }
        storage::auction_set_status(&env, id, types::AuctionStatus::Closed);
    }

    pub fn claim(env: Env, id: u32, claimant: Address) {
        claimant.require_auth();
        let status = storage::auction_get_status(&env, id);
        if status != types::AuctionStatus::Closed {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotClosed);
        }
        if storage::auction_is_claimed(&env, id) {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AlreadyClaimed);
        }
        let winner = storage::auction_get_highest_bidder(&env, id);
        if winner.as_ref().map(|w| w == &claimant).unwrap_or(false) {
            let asset = storage::auction_get_asset(&env, id);
            let token = soroban_sdk::token::Client::new(&env, &asset);
            let winning_bid = storage::auction_get_highest_bid(&env, id);
            let seller = storage::auction_get_seller(&env, id);
            token.transfer(&env.current_contract_address(), &seller, &winning_bid);
            storage::auction_set_claimed(&env, id);
        } else {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::NotWinner);
        }
    }
}
