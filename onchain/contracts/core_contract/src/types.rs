use soroban_sdk::{contracttype, Address, Bytes, BytesN, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct AddressMetadata {
    pub label: Symbol,
}

#[contracttype]
#[derive(Clone)]
pub struct ResolveData {
    pub wallet: Address,
    pub memo: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChainType {
    Evm,
    Bitcoin,
    Solana,
    Cosmos,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivacyMode {
    Normal,
    Shielded,
}

/// Serialized Groth16 proof bytes submitted by the caller.
pub type Proof = Bytes;

/// Public signals extracted from a Groth16 non-inclusion proof.
/// `old_root` must match the current on-chain SMT root.
/// `new_root` becomes the new SMT root after a successful registration.
#[contracttype]
#[derive(Clone)]
pub struct PublicSignals {
    pub commitment: BytesN<32>,
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
}
