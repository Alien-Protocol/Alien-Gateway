use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CoreError {
    /// The requested resource was not found.
    NotFound = 1,
    /// The SMT root has not been set yet.
    RootNotSet = 2,
    /// Commitment is already registered.
    DuplicateCommitment = 3,
    /// public_signals.old_root does not match the current on-chain SMT root.
    StaleRoot = 4,
    /// The supplied Groth16 proof is invalid.
    InvalidProof = 5,
    /// The username is registered but has no primary Stellar address linked.
    NoAddressLinked = 6,
    /// Caller is not the registered owner of the commitment.
    Unauthorized = 7,
    /// new_owner is the same as the current owner.
    SameOwner = 8,
    /// initialize() has already been called on this contract instance.
    AlreadyInitialized = 9,
    /// Commitment is already registered via register().
    AlreadyRegistered = 10,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChainAddressError {
    /// Caller is not the owner of the username commitment.
    Unauthorized = 1,
    /// The username commitment is not registered.
    NotRegistered = 2,
    /// The address format is invalid for the given chain type.
    InvalidAddress = 3,
}
