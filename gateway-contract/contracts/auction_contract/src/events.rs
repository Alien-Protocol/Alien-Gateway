use soroban_sdk::{Env, Address, BytesN, Symbol};

pub fn emit_username_claimed(env: &Env, username_hash: &BytesN<32>, claimer: &Address) {
    let topics = (Symbol::new(env, "USERNAME_CLAIMED"), username_hash.clone());
    env.events().publish(topics, claimer.clone());
}
