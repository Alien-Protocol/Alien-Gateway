use soroban_sdk::{contracttype, BytesN};
use soroban_sdk::{contracttype, Address, Symbol};

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
#[derive(Clone)]
pub struct ResolveData {
    pub wallet: Address,
    pub memo: Option<u64>,
}
