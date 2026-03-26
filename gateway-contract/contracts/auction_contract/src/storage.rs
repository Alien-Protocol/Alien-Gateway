//! Persistent storage helpers for the auction contract.
//!
//! This module owns the entire on-chain data layout for auctions and bids.
//! All reads and writes go through the functions below; nothing else in the
//! crate touches `env.storage()` directly.
//!
//! # Storage tiers
//!
//! Every key uses **persistent** storage so that auction and bid records
//! survive ledger archival.  Persistent entries must have their TTL extended
//! by the contract whenever they are read or written — see the individual
//! functions for their `extend_ttl` calls.
//!
//! # Key layout
//!
//! ```text
//! DataKey::Auction(hash)          → AuctionState
//! DataKey::Bid(hash, bidder)      → Bid
//! DataKey::AllBidders(hash)       → Vec<Address>
//! ```
//!
//! The `hash` in every key is the SHA-256 commitment to the auctioned asset's
//! metadata, produced off-chain and submitted when the auction is created.
//! Using a content-addressed key means the same asset cannot be auctioned
//! twice under different IDs without the creator supplying a different hash.

use soroban_sdk::{Address, BytesN, Env, Vec};

use crate::types::{AuctionState, Bid};

// ---------------------------------------------------------------------------
// TTL policy
// ---------------------------------------------------------------------------

/// Minimum number of ledgers an entry must remain live after it is touched.
///
/// Soroban charges rent proportional to the entry size × ledger duration.
/// Setting a generous threshold avoids frequent re-bumps while keeping rent
/// predictable.  Approximately 30 days at 5-second ledger close times.
const LEDGER_THRESHOLD: u32 = 518_400; // ~30 days

/// Target TTL to extend to on every touch (≈ 60 days).
const LEDGER_BUMP: u32 = 1_036_800;

// ---------------------------------------------------------------------------
// DataKey — the canonical key enum for this contract
// ---------------------------------------------------------------------------

/// All storage keys used by the auction contract.
///
/// Each variant maps one-to-one to a logical record type:
///
/// * `Auction(hash)` — the full [`AuctionState`] for one auction.
/// * `Bid(hash, bidder)` — the most-recent [`Bid`] from one address on one
///   auction.  A bidder can hold only one live bid per auction; placing a
///   second bid overwrites the first.
/// * `AllBidders(hash)` — the ordered list of distinct [`Address`] values
///   that have ever placed a bid on `hash`.  Used for iteration and refunds.
///
/// The `#[contracttype]` macro encodes each variant as a compact XDR value,
/// which becomes the raw storage key on-chain.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    /// Full auction state keyed by the asset commitment hash.
    Auction(BytesN<32>),
    /// One bidder's current bid on one auction.
    Bid(BytesN<32>, Address),
    /// Ordered list of all addresses that have bid on one auction.
    AllBidders(BytesN<32>),
}

// ---------------------------------------------------------------------------
// Auction helpers
// ---------------------------------------------------------------------------

/// Retrieve the [`AuctionState`] for `hash`, or `None` if it does not exist.
///
/// Extends the TTL on a hit so that active auctions are never archived while
/// they are being used.
///
/// # Arguments
/// * `env`  — the contract environment.
/// * `hash` — 32-byte commitment hash identifying the auction.
///
/// # Returns
/// `Some(AuctionState)` when found, `None` when the auction has never been
/// created or has been deleted.
pub fn get_auction(env: &Env, hash: &BytesN<32>) -> Option<AuctionState> {
    let key = DataKey::Auction(hash.clone());
    let result: Option<AuctionState> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    result
}

/// Persist `state` as the auction record for `hash`.
///
/// Overwrites any existing record for the same hash.  Callers are responsible
/// for ensuring the hash is not already in use for a different auction unless
/// an overwrite is intentional (e.g. status updates).
///
/// # Arguments
/// * `env`   — the contract environment.
/// * `hash`  — 32-byte commitment hash identifying the auction.
/// * `state` — the auction state to store.
pub fn set_auction(env: &Env, hash: &BytesN<32>, state: &AuctionState) {
    let key = DataKey::Auction(hash.clone());
    env.storage().persistent().set(&key, state);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Returns `true` if an [`AuctionState`] record exists for `hash`.
///
/// Does **not** extend the TTL — a pure existence check does not constitute
/// active use.  If the caller intends to read the record immediately after
/// this check, prefer `get_auction` which does both in one round-trip.
///
/// # Arguments
/// * `env`  — the contract environment.
/// * `hash` — 32-byte commitment hash identifying the auction.
pub fn has_auction(env: &Env, hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Auction(hash.clone()))
}

// ---------------------------------------------------------------------------
// Bid helpers
// ---------------------------------------------------------------------------

/// Retrieve the current [`Bid`] placed by `bidder` on auction `hash`, or
/// `None` if that bidder has not bid on this auction.
///
/// Extends the TTL on a hit.
///
/// # Arguments
/// * `env`    — the contract environment.
/// * `hash`   — 32-byte commitment hash of the auction.
/// * `bidder` — address of the bidder.
///
/// # Returns
/// `Some(Bid)` when the bidder has a live bid, `None` otherwise.
pub fn get_bid(env: &Env, hash: &BytesN<32>, bidder: &Address) -> Option<Bid> {
    let key = DataKey::Bid(hash.clone(), bidder.clone());
    let result: Option<Bid> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    result
}

/// Persist `bid` as the current bid from `bidder` on auction `hash`.
///
/// Any previously stored bid from the same `(hash, bidder)` pair is silently
/// overwritten.  The caller must update the bidder list via [`add_bidder`] if
/// this is the bidder's first bid on this auction.
///
/// # Arguments
/// * `env`    — the contract environment.
/// * `hash`   — 32-byte commitment hash of the auction.
/// * `bidder` — address of the bidder.
/// * `bid`    — the bid record to store.
pub fn set_bid(env: &Env, hash: &BytesN<32>, bidder: &Address, bid: &Bid) {
    let key = DataKey::Bid(hash.clone(), bidder.clone());
    env.storage().persistent().set(&key, bid);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Delete the [`Bid`] record for `bidder` on auction `hash`.
///
/// Called after a refund is issued so that the same bidder cannot be refunded
/// twice if `refund_losers` is invoked more than once.  A no-op when the key
/// does not exist.
///
/// # Arguments
/// * `env`    — the contract environment.
/// * `hash`   — 32-byte commitment hash of the auction.
/// * `bidder` — address whose bid record should be removed.
pub fn remove_bid(env: &Env, hash: &BytesN<32>, bidder: &Address) {
    let key = DataKey::Bid(hash.clone(), bidder.clone());
    env.storage().persistent().remove(&key);
}

// ---------------------------------------------------------------------------
// Bidder list helpers
// ---------------------------------------------------------------------------

/// Returns the ordered list of all addresses that have ever bid on auction
/// `hash`.
///
/// The returned `Vec` preserves insertion order — the first element is the
/// first bidder.  Duplicate addresses are never added; see [`add_bidder`].
///
/// Returns an empty `Vec` (not `None`) when no bids have been placed, so
/// callers can iterate unconditionally without an `Option` unwrap.
///
/// Extends the TTL on a non-empty result.
///
/// # Arguments
/// * `env`  — the contract environment.
/// * `hash` — 32-byte commitment hash of the auction.
pub fn get_all_bidders(env: &Env, hash: &BytesN<32>) -> Vec<Address> {
    let key = DataKey::AllBidders(hash.clone());
    let result: Vec<Address> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    if !result.is_empty() {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    result
}

/// Append `bidder` to the bidder list for auction `hash` if they are not
/// already present.
///
/// This function is idempotent — calling it multiple times with the same
/// `(hash, bidder)` pair is safe and adds the address at most once.
///
/// # Arguments
/// * `env`    — the contract environment.
/// * `hash`   — 32-byte commitment hash of the auction.
/// * `bidder` — address to add.
///
/// # Implementation note
/// The deduplication check is O(n) in the number of existing bidders.  This
/// is acceptable because the number of bidders per auction is bounded by
/// `MAX_BATCH_SIZE` enforced at the call site in `lib.rs`.
pub fn add_bidder(env: &Env, hash: &BytesN<32>, bidder: Address) {
    let key = DataKey::AllBidders(hash.clone());
    let mut bidders: Vec<Address> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    // Deduplication: only append if the address is not already in the list.
    for existing in bidders.iter() {
        if existing == bidder {
            return;
        }
    }

    bidders.push_back(bidder);
    env.storage().persistent().set(&key, &bidders);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
