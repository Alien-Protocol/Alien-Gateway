use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameClaimedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub claimer: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionClosedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub winner: Option<Address>,
    pub winning_bid: u128,
}

pub fn emit_username_claimed(env: &Env, username_hash: &BytesN<32>, claimer: &Address) {
    UsernameClaimedEvent {
        username_hash: username_hash.clone(),
        claimer: claimer.clone(),
    }
    .publish(env);
}

pub fn emit_auction_closed(
    env: &Env,
    username_hash: &BytesN<32>,
    winner: Option<Address>,
    winning_bid: u128,
) {
    AuctionClosedEvent {
        username_hash: username_hash.clone(),
        winner,
        winning_bid,
    }
    .publish(env);
}
