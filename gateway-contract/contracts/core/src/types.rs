#![no_std]
use soroban_sdk::{contracttype, String};

/// Represents an external chain identifier
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[contracttype]
pub enum ChainId {
    Ethereum,
    Bitcoin,
    Solana,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
}

impl ChainId {
    pub fn to_string(&self) -> &'static str {
        match self {
            ChainId::Ethereum => "ethereum",
            ChainId::Bitcoin => "bitcoin",
            ChainId::Solana => "solana",
            ChainId::Polygon => "polygon",
            ChainId::Arbitrum => "arbitrum",
            ChainId::Optimism => "optimism",
            ChainId::Base => "base",
        }
    }
}

/// Represents a stored address on an external chain
#[derive(Clone)]
#[contracttype]
pub struct ChainAddress {
    pub chain: ChainId,
    pub address: String,
    pub label: String,
}

/// Events emitted by the chain registry contract
#[derive(Clone)]
#[contracttype]
pub enum ChainRegistryEvent {
    /// Emitted when a new chain address is added
    ChainAddressAdded {
        chain: ChainId,
        address: String,
        label: String,
    },
    /// Emitted when a chain address is removed
    ChainAddressRemoved {
        chain: ChainId,
        address: String,
    },
    /// Emitted when owner is changed
    OwnerChanged {
        new_owner: soroban_sdk::Address,
    },
}
