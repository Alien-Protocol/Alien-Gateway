use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    RootMismatch = 3,
    InvalidProof = 4,
    DuplicateCommitment = 5,
}
