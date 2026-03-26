use soroban_sdk::{contractevent, BytesN};

#[contractevent]
pub struct UsernameRegistered {
    pub commitment: BytesN<32>,
}
