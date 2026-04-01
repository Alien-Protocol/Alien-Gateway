use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameRecord {
    pub username_hash: BytesN<32>,
    pub owner: Address,
    pub registered_at: u64,
    pub core_contract: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeployConfig {
    pub core_contract_wasm_hash: BytesN<32>,
    pub admin: Address,
}
