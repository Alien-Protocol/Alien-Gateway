use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::types::UsernameRecord;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    AuctionContract,
    CoreContract,
    Username(BytesN<32>),
}

pub fn set_auction_contract(env: &Env, auction_contract: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::AuctionContract, auction_contract);
}

pub fn get_auction_contract(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get::<DataKey, Address>(&DataKey::AuctionContract)
}

pub fn set_core_contract(env: &Env, core_contract: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::CoreContract, core_contract);
}

pub fn get_core_contract(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get::<DataKey, Address>(&DataKey::CoreContract)
}

pub fn set_username_record(env: &Env, record: &UsernameRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Username(record.username_hash.clone()), record);
}

pub fn get_username_record(env: &Env, username_hash: &BytesN<32>) -> Option<UsernameRecord> {
    env.storage()
        .persistent()
        .get::<DataKey, UsernameRecord>(&DataKey::Username(username_hash.clone()))
}
