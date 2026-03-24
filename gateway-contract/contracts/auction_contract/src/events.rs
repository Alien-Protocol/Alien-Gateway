use soroban_sdk::{contracttype, Address, Env, Symbol};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct BidPlacedEvent {
    pub id: u32,
    pub bidder: Address,
    pub amount: i128,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ClaimedEvent {
    pub id: u32,
    pub claimant: Address,
}

pub struct Events;

impl Events {
    pub fn bid_placed(env: &Env, id: u32, bidder: Address, amount: i128) {
        env.events().publish(
            (Symbol::new(env, "bid_placed"), id),
            BidPlacedEvent { id, bidder, amount }
        );
    }

    pub fn claimed(env: &Env, id: u32, claimant: Address) {
        env.events().publish(
            (Symbol::new(env, "claimed"), id),
            ClaimedEvent { id, claimant }
        );
    }
}