#![no_std]

mod events;

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

use events::publish_bid_refunded_event;

#[contract]
pub struct Auction;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionState {
    pub bidder: Address,
    pub amount: i128,
}

#[contractimpl]
impl Auction {
    /// Place a bid for an auction identified by `auction_id`.
    /// If there's a previous highest bidder, emit a `BID_RFDN` event
    /// before attempting the refund token transfer.
    pub fn place_bid(env: Env, auction_id: Symbol, bidder: Address, amount: i128) {
        bidder.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Load existing highest bid if any
        let existing: Option<AuctionState> = env.storage().persistent().get(&auction_id);

        if let Some(prev) = existing {
            if amount <= prev.amount {
                panic!("bid must be higher than current highest bid");
            }

            // Emit refund event before performing token transfer
            publish_bid_refunded_event(&env, prev.bidder.clone(), prev.amount);

            // Attempt refund token transfer if token address configured in instance storage
            let token_addr: Option<Address> = env.storage().instance().get(&Symbol::new(&env, "bid_token"));
            if let Some(tkn) = token_addr {
                let token_client = token::Client::new(&env, &tkn);
                // Contract is the sender of refund transfers (for tests this will be mocked)
                token_client.transfer(&env.current_contract_address(), &prev.bidder, &prev.amount);
            }
        }

        // Store new highest bid
        let new_state = AuctionState { bidder: bidder.clone(), amount };
        env.storage().persistent().set(&auction_id, &new_state);
    }
}

#[cfg(test)]
mod test;
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
        if end_time <= env.ledger().timestamp() {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotClosed);
        }
        storage::auction_set_seller(&env, id, &seller);
        storage::auction_set_asset(&env, id, &asset);
        storage::auction_set_min_bid(&env, id, min_bid);
        storage::auction_set_end_time(&env, id, end_time);
        storage::auction_set_status(&env, id, types::AuctionStatus::Open);
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
            storage::auction_set_outbid_amount(&env, id, &prev_bidder, existing_outbid + prev_amount);
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
        if highest_bidder.as_ref().map(|h| h == &bidder).unwrap_or(false) {
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
        events::emit_bid_refunded(&env, &BytesN::from_array(&env, &[0u8; 32]), &bidder, refund_amount);
    }

    pub fn close_auction_by_id(env: Env, id: u32) {
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
