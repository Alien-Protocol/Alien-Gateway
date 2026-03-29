use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
/// Errors returned by the factory contract.
pub enum FactoryError {
    /// Caller is not the configured auction contract.
    Unauthorized = 1,
    /// Username hash is already registered.
    AlreadyDeployed = 2,
    /// Core contract address has not been configured.
    CoreContractNotConfigured = 3,
}
