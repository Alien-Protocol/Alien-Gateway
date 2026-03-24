use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    /// The contract has already been initialized and cannot be initialized again.
    AlreadyInitialized = 1,
    /// The caller is not authorized to perform this operation.
    Unauthorized = 2,
    /// The provided contract hash is invalid or not found.
    InvalidContractHash = 3,
    /// Contract deployment failed.
    DeploymentFailed = 4,
}
