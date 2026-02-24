use soroban_sdk::{contracttype, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct AddressMetadata {
    pub label: Symbol,
}
