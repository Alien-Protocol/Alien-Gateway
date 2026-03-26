use soroban_sdk::{contracttype, BytesN};

/// Storage keys for the Core contract's persistent and instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Key for resolver data, indexed by commitment.
    Resolver(BytesN<32>),
    /// Key for the SMT root in instance storage.
    SmtRoot,
    /// Key for the primary Stellar address linked to a username hash.
    StellarAddress(BytesN<32>),
}
