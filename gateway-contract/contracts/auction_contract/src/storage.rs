use crate::types::{AuctionStatus, DataKey};
use soroban_sdk::{Address, Env, u128};

pub fn get_status(env: &Env) -> AuctionStatus {
    env.storage()
        .instance()
        .get(&DataKey::Status)
        .unwrap_or(AuctionStatus::Open)
}

pub fn set_status(env: &Env, status: AuctionStatus) {
    env.storage().instance().set(&DataKey::Status, &status);
}

pub fn get_highest_bidder(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::HighestBidder)
}

pub fn set_highest_bidder(env: &Env, bidder: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::HighestBidder, bidder);
}

pub fn get_factory_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::FactoryContract)
}

pub fn set_factory_contract(env: &Env, factory: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::FactoryContract, factory);
}

pub fn get_end_time(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::EndTime)
        .unwrap_or(0)
}

pub fn set_end_time(env: &Env, end_time: u64) {
    env.storage().instance().set(&DataKey::EndTime, &end_time);
}

pub fn get_highest_bid(env: &Env) -> u128 {
    env.storage()
        .instance()
        .get(&DataKey::HighestBid)
        .unwrap_or(0)
}

pub fn set_highest_bid(env: &Env, bid: u128) {
    env.storage().instance().set(&DataKey::HighestBid, &bid);
}
