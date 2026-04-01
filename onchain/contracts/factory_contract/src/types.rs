use soroban_sdk::{contracttype, Address, BytesN};

/// Storage keys used by the factory contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Address of the auction contract authorised to deploy usernames.
    AuctionContract,
    /// Address of the core contract associated with new usernames.
    CoreContract,
    /// Record for a registered username, keyed by its 32-byte hash.
    Username(BytesN<32>),
    /// Optional deployment configuration for the factory.
    Config,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// On-chain record for a registered username.
pub struct UsernameRecord {
    /// 32-byte hash that uniquely identifies the username.
    pub username_hash: BytesN<32>,
    /// Address that owns this username.
    pub owner: Address,
    /// Ledger timestamp at which the username was registered.
    pub registered_at: u64,
    /// Core contract address associated with this username.
    pub core_contract: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Configuration required to deploy a new core contract instance.
pub struct DeployConfig {
    /// WASM hash of the core contract to be instantiated.
    pub core_contract_wasm_hash: BytesN<32>,
    /// Admin address authorised to manage the factory.
    pub admin: Address,
}
