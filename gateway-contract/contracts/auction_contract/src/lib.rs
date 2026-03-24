#![no_std]

mod storage;
mod types;

pub use storage::{
    add_bidder, get_all_bidders, get_auction, get_bid, has_auction, set_auction, set_bid, DataKey,
};
pub use types::{AuctionState, Bid};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    // -----------------------------------------------------------------------
    // Auction management
    // -----------------------------------------------------------------------

    /// Create a new auction identified by `hash`.
    ///
    /// The `hash` must not already be in use — callers should verify with
    /// `has_auction` before calling this.  The creator must authorise the
    /// call.
    ///
    /// # Errors
    /// Panics with `"auction already exists"` when `hash` is already in use.
    pub fn create_auction(
        env: Env,
        creator: Address,
        hash: BytesN<32>,
        start_time: u64,
        end_time: u64,
        reserve_price: i128,
    ) {
        creator.require_auth();
        assert!(!has_auction(&env, &hash), "auction already exists");

        let state = AuctionState {
            creator,
            start_time,
            end_time,
            reserve_price,
            highest_bid: 0,
            highest_bidder: None,
            is_settled: false,
        };
        set_auction(&env, &hash, &state);
    }

    /// View: returns the full [`AuctionState`] for `hash`.
    ///
    /// # Errors
    /// Panics with `"auction not found"` when `hash` has no associated state.
    pub fn get_auction(env: Env, hash: BytesN<32>) -> AuctionState {
        get_auction(&env, &hash).expect("auction not found")
    }

    /// View: returns `true` if an auction exists for `hash`.
    pub fn has_auction(env: Env, hash: BytesN<32>) -> bool {
        has_auction(&env, &hash)
    }

    // -----------------------------------------------------------------------
    // Bidding
    // -----------------------------------------------------------------------

    /// Place or update a bid from `bidder` on auction `hash`.
    ///
    /// * The auction must exist.
    /// * `amount` must exceed the current `highest_bid`.
    /// * `bidder` must authorise the call.
    ///
    /// # Errors
    /// Panics on constraint violations (auction missing, bid too low).
    pub fn place_bid(env: Env, hash: BytesN<32>, bidder: Address, amount: i128) {
        bidder.require_auth();

        let mut state = get_auction(&env, &hash).expect("auction not found");
        assert!(amount > state.highest_bid, "bid must exceed highest bid");

        let bid = Bid {
            bidder: bidder.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
        };

        // Update the bidder list before writing the bid record so that the
        // AllBidders key is always at least as fresh as any Bid key.
        add_bidder(&env, &hash, bidder.clone());
        set_bid(&env, &hash, &bidder, &bid);

        state.highest_bid = amount;
        state.highest_bidder = Some(bidder);
        set_auction(&env, &hash, &state);
    }

    /// View: returns the [`Bid`] placed by `bidder` on `hash`, if any.
    pub fn get_bid(env: Env, hash: BytesN<32>, bidder: Address) -> Option<Bid> {
        get_bid(&env, &hash, &bidder)
    }

    /// View: returns all addresses that have bid on `hash`.
    pub fn get_all_bidders(env: Env, hash: BytesN<32>) -> Vec<Address> {
        get_all_bidders(&env, &hash)
    }
}
