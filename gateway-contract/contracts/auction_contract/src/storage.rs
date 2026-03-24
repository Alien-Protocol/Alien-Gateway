use crate::types::Auction;
use soroban_sdk::{Env, Symbol};

pub struct Storage;

impl Storage {
    pub fn set_auction(env: &Env, _id: u32, auction: &Auction) {
        env.storage()
            .instance()
            .set(&Symbol::new(env, "auction"), auction);
    }

    pub fn get_auction(env: &Env, __id: u32) -> Auction {
        env.storage()
            .instance()
            .get(&Symbol::new(env, "auction"))
            .unwrap()
    }

    pub fn has_auction(env: &Env, __id: u32) -> bool {
        env.storage().instance().has(&Symbol::new(env, "auction"))
    }
}
