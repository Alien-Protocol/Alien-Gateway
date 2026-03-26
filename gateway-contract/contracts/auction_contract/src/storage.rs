use crate::types::{AuctionStatus, DataKey};
use soroban_sdk::{Address, Env};

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
    env.storage().instance().get(&DataKey::EndTime).unwrap_or(0)
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

// --- id-scoped auction storage ---
use crate::types::AuctionKey;

pub fn auction_exists(env: &Env, id: u32) -> bool {
    env.storage().persistent().has(&AuctionKey::Status(id))
}

pub fn auction_get_status(env: &Env, id: u32) -> crate::types::AuctionStatus {
    env.storage()
        .persistent()
        .get(&AuctionKey::Status(id))
        .unwrap_or(crate::types::AuctionStatus::Open)
}

pub fn auction_set_status(env: &Env, id: u32, status: crate::types::AuctionStatus) {
    env.storage()
        .persistent()
        .set(&AuctionKey::Status(id), &status);
}

pub fn auction_get_seller(env: &Env, id: u32) -> Address {
    env.storage()
        .persistent()
        .get(&AuctionKey::Seller(id))
        .unwrap()
}

pub fn auction_set_seller(env: &Env, id: u32, seller: &Address) {
    env.storage()
        .persistent()
        .set(&AuctionKey::Seller(id), seller);
}

pub fn auction_get_asset(env: &Env, id: u32) -> Address {
    env.storage()
        .persistent()
        .get(&AuctionKey::Asset(id))
        .unwrap()
}

pub fn auction_set_asset(env: &Env, id: u32, asset: &Address) {
    env.storage()
        .persistent()
        .set(&AuctionKey::Asset(id), asset);
}

pub fn auction_get_min_bid(env: &Env, id: u32) -> i128 {
    env.storage()
        .persistent()
        .get(&AuctionKey::MinBid(id))
        .unwrap_or(0)
}

pub fn auction_set_min_bid(env: &Env, id: u32, min_bid: i128) {
    env.storage()
        .persistent()
        .set(&AuctionKey::MinBid(id), &min_bid);
}

pub fn auction_get_end_time(env: &Env, id: u32) -> u64 {
    env.storage()
        .persistent()
        .get(&AuctionKey::EndTime(id))
        .unwrap_or(0)
}

pub fn auction_set_end_time(env: &Env, id: u32, end_time: u64) {
    env.storage()
        .persistent()
        .set(&AuctionKey::EndTime(id), &end_time);
}

pub fn auction_get_highest_bidder(env: &Env, id: u32) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&AuctionKey::HighestBidder(id))
}

pub fn auction_set_highest_bidder(env: &Env, id: u32, bidder: &Address) {
    env.storage()
        .persistent()
        .set(&AuctionKey::HighestBidder(id), bidder);
}

pub fn auction_get_highest_bid(env: &Env, id: u32) -> i128 {
    env.storage()
        .persistent()
        .get(&AuctionKey::HighestBid(id))
        .unwrap_or(0)
}

pub fn auction_set_highest_bid(env: &Env, id: u32, bid: i128) {
    env.storage()
        .persistent()
        .set(&AuctionKey::HighestBid(id), &bid);
}

pub fn auction_is_claimed(env: &Env, id: u32) -> bool {
    env.storage()
        .persistent()
        .get(&AuctionKey::Claimed(id))
        .unwrap_or(false)
}

pub fn auction_set_claimed(env: &Env, id: u32) {
    env.storage()
        .persistent()
        .set(&AuctionKey::Claimed(id), &true);
}
