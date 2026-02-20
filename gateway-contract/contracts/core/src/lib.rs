#![no_std]

pub mod chain_registry;
pub mod types;

#[cfg(test)]
mod tests;

pub use chain_registry::ChainRegistry;
pub use types::{ChainAddress, ChainId, ChainRegistryEvent};
