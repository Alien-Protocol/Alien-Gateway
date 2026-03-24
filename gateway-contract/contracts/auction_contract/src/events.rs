use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameClaimedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub claimer: Address,
}

pub fn emit_username_claimed(env: &Env, username_hash: &BytesN<32>, claimer: &Address) {
    UsernameClaimedEvent {
        username_hash: username_hash.clone(),
        claimer: claimer.clone(),
    }
    .publish(env);
}
