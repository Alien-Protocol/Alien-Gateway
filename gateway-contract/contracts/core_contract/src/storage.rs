use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Root,
    Verifier,
    Commitment(BytesN<32>),
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Root) && env.storage().instance().has(&DataKey::Verifier)
}

pub fn get_root(env: &Env) -> Option<BytesN<32>> {
    env.storage().instance().get(&DataKey::Root)
}

pub fn set_root(env: &Env, root: &BytesN<32>) {
    env.storage().instance().set(&DataKey::Root, root);
}

pub fn get_verifier(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Verifier)
}

pub fn set_verifier(env: &Env, verifier: &Address) {
    env.storage().instance().set(&DataKey::Verifier, verifier);
}

pub fn has_commitment(env: &Env, commitment: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Commitment(commitment.clone()))
}

pub fn store_commitment(env: &Env, commitment: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::Commitment(commitment.clone()), &true);
}
