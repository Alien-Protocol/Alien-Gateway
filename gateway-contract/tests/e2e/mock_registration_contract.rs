use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
pub struct MockRegistrationContract;

#[contractimpl]
impl MockRegistrationContract {
    pub fn set_owner(env: Env, commitment: BytesN<32>, owner: Address) {
        env.storage().persistent().set(&commitment, &owner);
    }
    pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address> {
        env.storage().persistent().get(&commitment)
    }
}
