use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    Unauthorized = 1,
    AlreadyDeployed = 2,
    CoreContractNotConfigured = 3,
}
