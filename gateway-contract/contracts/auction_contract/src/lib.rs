#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Address, BytesN, Env, IntoVal, Symbol};

pub mod errors;
pub mod events;
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
/// Auction contract for the Alien Gateway username system.
///
/// Manages auction lifecycle (create, bid, close, claim) and
/// username claiming via a factory contract.
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    /// Closes the global auction after its end time has passed.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `username_hash` – The 32-byte hash of the auctioned username.
    ///
    /// # Errors
    /// - [`errors::AuctionError::AuctionNotOpen`] – The auction status is not `Open`.
    /// - [`errors::AuctionError::AuctionNotClosed`] – The current ledger timestamp is before `end_time`.
    ///
    /// # State Changes
    /// Sets the auction status to `Closed` and emits an `AUCTION_CLOSED` event
    /// containing the winner address and winning bid amount.
    pub fn close_auction(
        env: Env,
        username_hash: BytesN<32>,
    ) -> Result<(), crate::errors::AuctionError> {
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

    /// Claims the auctioned username after the auction is closed.
    ///
    /// Deploys the username via the factory contract so the winner can use it
    /// as their Stellar identity.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `username_hash` – The 32-byte hash of the auctioned username.
    /// - `claimer` – The address of the caller (must be the highest bidder).
    ///
    /// # Authorization
    /// Requires `claimer.require_auth()`.
    ///
    /// # Errors
    /// - [`errors::AuctionError::AlreadyClaimed`] – The username has already been claimed.
    /// - [`errors::AuctionError::NotClosed`] – The auction is not yet closed.
    /// - [`errors::AuctionError::NotWinner`] – The claimer is not the highest bidder.
    /// - [`errors::AuctionError::NoFactoryContract`] – No factory contract address is configured.
    ///
    /// # State Changes
    /// Sets the auction status to `Claimed`, invokes `deploy_username` on the
    /// factory contract, and emits a `USERNAME_CLAIMED` event.
    pub fn claim_username(
        env: Env,
        username_hash: BytesN<32>,
        claimer: Address,
    ) -> Result<(), crate::errors::AuctionError> {
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
    /// Creates a new auction with the given parameters.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `id` – Unique auction identifier.
    /// - `seller` – The address creating the auction (must authorize).
    /// - `asset` – The token contract address used for bidding.
    /// - `min_bid` – The minimum accepted bid amount.
    /// - `end_time` – Ledger timestamp after which the auction can be closed.
    ///
    /// # Authorization
    /// Requires `seller.require_auth()`.
    ///
    /// # Errors
    /// Panics with [`errors::AuctionError::AuctionNotOpen`] if an auction with `id` already exists.
    ///
    /// # State Changes
    /// Stores the seller, asset, min bid, end time, and sets the auction status to `Open`.
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
        storage::auction_set_seller(&env, id, &seller);
        storage::auction_set_asset(&env, id, &asset);
        storage::auction_set_min_bid(&env, id, min_bid);
        storage::auction_set_end_time(&env, id, end_time);
        storage::auction_set_status(&env, id, types::AuctionStatus::Open);
    }

    /// Places a bid on an open auction.
    ///
    /// Transfers the bid amount from the bidder to the contract. If a previous
    /// highest bid exists, it is refunded to the previous bidder.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `id` – The auction identifier.
    /// - `bidder` – The address placing the bid (must authorize).
    /// - `amount` – The bid amount (must exceed both `min_bid` and the current highest bid).
    ///
    /// # Authorization
    /// Requires `bidder.require_auth()`.
    ///
    /// # Errors
    /// - Panics with [`errors::AuctionError::AuctionNotOpen`] if the auction has ended.
    /// - Panics with [`errors::AuctionError::BidTooLow`] if `amount` is below `min_bid` or not
    ///   greater than the current highest bid.
    ///
    /// # State Changes
    /// Updates the highest bidder and highest bid. Transfers tokens from the bidder
    /// to the contract and refunds the previous highest bidder if any.
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
        let asset = storage::auction_get_asset(&env, id);
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&bidder, env.current_contract_address(), &amount);
        if let Some(prev_bidder) = storage::auction_get_highest_bidder(&env, id) {
            token.transfer(&env.current_contract_address(), &prev_bidder, &highest_bid);
        }
        storage::auction_set_highest_bidder(&env, id, &bidder);
        storage::auction_set_highest_bid(&env, id, amount);
    }

    /// Closes an auction by its numeric identifier after the end time has passed.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `id` – The auction identifier.
    ///
    /// # Errors
    /// Panics with [`errors::AuctionError::AuctionNotClosed`] if the current ledger timestamp
    /// is before the auction's end time.
    ///
    /// # State Changes
    /// Sets the auction status to `Closed`.
    pub fn close_auction_by_id(env: Env, id: u32) {
        let end_time = storage::auction_get_end_time(&env, id);
        if env.ledger().timestamp() < end_time {
            soroban_sdk::panic_with_error!(&env, errors::AuctionError::AuctionNotClosed);
        }
        storage::auction_set_status(&env, id, types::AuctionStatus::Closed);
    }

    /// Claims the proceeds of a closed auction.
    ///
    /// The winning bidder calls this to transfer the winning bid amount from the
    /// contract to the seller.
    ///
    /// # Parameters
    /// - `env` – The contract environment.
    /// - `id` – The auction identifier.
    /// - `claimant` – The address claiming the auction (must be the highest bidder).
    ///
    /// # Authorization
    /// Requires `claimant.require_auth()`.
    ///
    /// # Errors
    /// - Panics with [`errors::AuctionError::NotClosed`] if the auction is not in `Closed` status.
    /// - Panics with [`errors::AuctionError::AlreadyClaimed`] if the auction has already been claimed.
    /// - Panics with [`errors::AuctionError::NotWinner`] if the claimant is not the highest bidder.
    ///
    /// # State Changes
    /// Transfers the winning bid from the contract to the seller and marks the
    /// auction as claimed.
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
