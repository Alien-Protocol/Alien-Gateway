use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::types::{DeployConfig, UsernameRecord};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    AuctionContract,
    CoreContract,
    Username(BytesN<32>),
    Config,
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

pub fn set_username(env: &Env, hash: &BytesN<32>, record: &UsernameRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Username(hash.clone()), record);
}

pub fn get_username(env: &Env, hash: &BytesN<32>) -> Option<UsernameRecord> {
    env.storage()
        .persistent()
        .get::<DataKey, UsernameRecord>(&DataKey::Username(hash.clone()))
}

pub fn has_username(env: &Env, hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Username(hash.clone()))
}

#[allow(dead_code)]
pub fn get_config(env: &Env) -> Option<DeployConfig> {
    env.storage()
        .persistent()
        .get::<DataKey, DeployConfig>(&DataKey::Config)
}

#[allow(dead_code)]
pub fn set_config(env: &Env, config: &DeployConfig) {
    env.storage().persistent().set(&DataKey::Config, config);
}
