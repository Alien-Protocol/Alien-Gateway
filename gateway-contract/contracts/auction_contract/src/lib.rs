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
/// Auction contract for the Alien Gateway username system.
///
/// Manages auction lifecycle (create, bid, close, claim) and
/// username claiming via a factory contract.
pub struct AuctionContract;

/// Singleton flow: one auction per contract instance.
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
    ) -> Result<(), errors::AuctionError> {
        singleton::close_auction(&env, username_hash)
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
    ) -> Result<(), errors::AuctionError> {
        singleton::claim_username(&env, username_hash, claimer)
    }
}

/// ID-indexed flow: multiple auctions identified by a numeric id.
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
        indexed::create_auction(&env, id, seller, asset, min_bid, end_time)
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
        indexed::place_bid(&env, id, bidder, amount)
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
        indexed::close_auction_by_id(&env, id)
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
        indexed::claim(&env, id, claimant)
    }
}
