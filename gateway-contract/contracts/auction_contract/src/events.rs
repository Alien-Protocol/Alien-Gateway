use soroban_sdk::{contractevent, symbol_short, Address, BytesN, Env, Symbol};

// Event symbols must be <= 9 chars (Soroban `symbol_short!`).
pub const AUCTION_CREATED: Symbol = symbol_short!("AUCR_CRT");
pub const BID_PLACED: Symbol = symbol_short!("BID_PLCD");
pub const AUCTION_CLOSED: Symbol = symbol_short!("AUCR_CLSD");
pub const USERNAME_CLAIMED: Symbol = symbol_short!("USR_CLMD");
pub const BID_REFUNDED: Symbol = symbol_short!("BID_RFDN");

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionCreatedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub end_time: u64,
    pub min_bid: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BidPlacedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub bidder: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionClosedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub winner: Address,
    pub winning_bid: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameClaimedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub owner: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BidRefundedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub bidder: Address,
    pub refund_amount: i128,
}

pub fn emit_auction_created(env: &Env, username_hash: &BytesN<32>, end_time: u64, min_bid: i128) {
    AuctionCreatedEvent {
        username_hash: username_hash.clone(),
        end_time,
        min_bid,
    }
    .publish(env);
}

pub fn emit_bid_placed(env: &Env, username_hash: &BytesN<32>, bidder: &Address, amount: i128) {
    BidPlacedEvent {
        username_hash: username_hash.clone(),
        bidder: bidder.clone(),
        amount,
    }
    .publish(env);
}

pub fn emit_auction_closed(
    env: &Env,
    username_hash: &BytesN<32>,
    winner: &Address,
    winning_bid: i128,
) {
    AuctionClosedEvent {
        username_hash: username_hash.clone(),
        winner: winner.clone(),
        winning_bid,
    }
    .publish(env);
}

pub fn emit_username_claimed(env: &Env, username_hash: &BytesN<32>, owner: &Address) {
    UsernameClaimedEvent {
        username_hash: username_hash.clone(),
        owner: owner.clone(),
    }
    .publish(env);
}

pub fn emit_bid_refunded(
    env: &Env,
    username_hash: &BytesN<32>,
    bidder: &Address,
    refund_amount: i128,
) {
    BidRefundedEvent {
        username_hash: username_hash.clone(),
        bidder: bidder.clone(),
        refund_amount,
    }
    .publish(env);
}
