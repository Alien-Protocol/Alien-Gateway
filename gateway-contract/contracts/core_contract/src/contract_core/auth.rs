use soroban_sdk::{symbol_short, Address, Env};

/// Storage key for the contract owner.
const OWNER: soroban_sdk::Symbol = symbol_short!("Owner");

/// Read the owner address from instance storage.
/// Returns `None` if the contract has not been initialized yet.
pub fn get_owner(env: &Env) -> Option<Address> {
    env.storage().instance().get(&OWNER)
}

/// Require that the contract has been initialized and that the calling
/// transaction has been authorized by the stored owner.
///
/// Panics with `"Contract not initialized"` if no owner is stored.
/// Delegates to `Address::require_auth` for the authorization check,
/// which will panic with a Soroban auth error if the caller is not the owner.
pub fn require_owner(env: &Env) {
    let owner: Address = env
        .storage()
        .instance()
        .get(&OWNER)
        .expect("Contract not initialized");
    owner.require_auth();
}
