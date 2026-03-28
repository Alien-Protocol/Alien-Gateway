use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};

/// The complete on-chain state of one auction.
///
/// An auction is uniquely identified by its `hash` (`BytesN<32>`), which is
/// used as the key in `DataKey::Auction(hash)`.  The hash is typically the
/// SHA-256 of the auctioned-asset metadata committed off-chain.
///
/// # Status transitions
///
/// ```text
/// Created ──► Active ──► Ended ──► Settled
///                  └──► Cancelled
/// ```
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionState {
    /// Address that created and owns this auction.
    pub creator: Address,
    /// Unix timestamp (seconds) at which bidding opens.
    pub start_time: u64,
    /// Unix timestamp (seconds) at which bidding closes.
    pub end_time: u64,
    /// Minimum bid accepted in the token's base units.
    pub reserve_price: i128,
    /// Highest bid seen so far; `0` when no bids have been placed.
    pub highest_bid: i128,
    /// Address of the current highest bidder; `None` when no bids placed.
    pub highest_bidder: Option<Address>,
    /// Whether the auction creator has closed the auction.
    pub is_settled: bool,
}

/// A single bid placed by one address on one auction.
///
/// Stored under `DataKey::Bid(auction_hash, bidder)`.
/// Each bidder can hold exactly one active bid per auction; placing a new bid
/// overwrites the previous record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bid {
    /// The bidder's address (redundant but useful for event payloads).
    pub bidder: Address,
    /// Bid amount in the token's base units.
    pub amount: i128,
    /// Ledger timestamp at which this bid was accepted.
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AuctionConfig {
    pub username_hash: BytesN<32>,
    pub start_time: u64,
    pub end_time: u64,
    pub min_bid: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct AuctionState {
    pub config: AuctionConfig,
    pub status: AuctionStatus,
    pub highest_bidder: Option<Address>,
    pub highest_bid: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
    pub timestamp: u64,
}
