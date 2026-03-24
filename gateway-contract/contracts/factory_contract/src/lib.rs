#![no_std]
use soroban_sdk::{contract, contractimpl};

mod errors;
pub use errors::FactoryError;

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {}
