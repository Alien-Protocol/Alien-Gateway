use soroban_sdk::{contracttype, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub a: BytesN<32>,
    pub b: BytesN<32>,
    pub c: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicSignals {
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
    pub commitment: BytesN<32>,
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
    Private,
}

/// Public signals extracted from a Groth16 non-inclusion proof.
/// `old_root` must match the current on-chain SMT root.
/// `new_root` becomes the new SMT root after a successful registration.
#[contracttype]
#[derive(Clone)]
pub struct PublicSignals {
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
}
