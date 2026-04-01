use soroban_sdk::{panic_with_error, Address, Bytes, BytesN, Env};

use crate::errors::CoreError;
use crate::events::{privacy_set_event, REGISTER_EVENT};
use crate::registration::Registration;
use crate::storage;
use crate::types::{PrivacyMode, PublicSignals, ResolveData};
use crate::{smt_root, zk_verifier};

pub struct Resolver;

impl Resolver {
    /// Registers a username with zero-knowledge proof validation.
    ///
    /// This advanced registration validates a ZK proof that the username hasn't been used before
    /// on the SMT. The proof must be valid against the current SMT root. Upon successful verification,
    /// the new root is updated. The caller must authorize this transaction.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `caller`: The address registering the commitment. Must be authorized.
    /// - `commitment`: A 32-byte Poseidon hash of the username.
    /// - `proof`: Serialized Groth16 proof for non-inclusion in the SMT.
    /// - `public_signals`: Public inputs including old_root, new_root, and commitment.
    ///
    /// ### Errors
    /// - `DuplicateCommitment`: If the commitment is already registered.
    /// - `RootNotSet`: If the SMT root has not been initialized.
    /// - `StaleRoot`: If the proof's old_root doesn't match the current SMT root.
    /// - `InvalidProof`: If the ZK proof verification fails.
    ///
    /// ### Events
    /// - Emits `REGISTER_EVENT` with (commitment, caller).
    /// - Updates the SMT root via `ROOT_UPDATED` event.
    pub fn register_resolver(
        env: Env,
        caller: Address,
        commitment: BytesN<32>,
        proof: Bytes,
        public_signals: PublicSignals,
    ) {
        caller.require_auth();

        let key = storage::DataKey::Resolver(commitment.clone());
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, CoreError::DuplicateCommitment);
        }

        let current_root = smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet));
        if public_signals.old_root != current_root {
            panic_with_error!(&env, CoreError::StaleRoot);
        }

        if public_signals.commitment != commitment {
            panic_with_error!(&env, CoreError::InvalidProof);
        }

        if !zk_verifier::ZkVerifier::verify_groth16_proof(&env, &proof, &public_signals) {
            panic_with_error!(&env, CoreError::InvalidProof);
        }

        let data = ResolveData {
            wallet: caller.clone(),
            memo: None,
        };
        env.storage().persistent().set(&key, &data);

        smt_root::SmtRoot::update_root(&env, public_signals.new_root);

        #[allow(deprecated)]
        env.events()
            .publish((REGISTER_EVENT,), (commitment, caller));
    }

    /// Sets a memo field for a registered commitment.
    ///
    /// Associates a 64-bit memo ID with a username commitment. The memo can be used to link
    /// external payment identifiers or metadata. This updates the resolver data for the commitment.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `commitment`: The 32-byte username commitment.
    /// - `memo_id`: A 64-bit unsigned integer memo value.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    pub fn set_memo(env: Env, commitment: BytesN<32>, memo_id: u64) {
        let mut data = env
            .storage()
            .persistent()
            .get::<storage::DataKey, ResolveData>(&storage::DataKey::Resolver(commitment.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));

        data.memo = Some(memo_id);
        env.storage()
            .persistent()
            .set(&storage::DataKey::Resolver(commitment), &data);
    }

    /// Sets the privacy mode for a commitment (Normal or Shielded).
    ///
    /// Determines whether the commitment resolves to the actual wallet address (Normal) or
    /// to the contract address (Shielded). Only the commitment owner can authorize this change.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    /// - `mode`: The privacy mode (`Normal` or `Shielded`).
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered or has no owner.
    ///
    /// ### Events
    /// - Emits `PRIVACY_SET` event with (username_hash, mode).
    pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode) {
        let owner = Registration::get_owner(env.clone(), username_hash.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        owner.require_auth();

        storage::set_privacy_mode(&env, &username_hash, &mode);

        #[allow(deprecated)]
        env.events()
            .publish((privacy_set_event(&env),), (username_hash, mode));
    }

    /// Retrieves the privacy mode for a commitment.
    ///
    /// Returns the current privacy mode (Normal or Shielded) for the given commitment.
    /// Defaults to `Normal` if not explicitly set. This is a read-only operation.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `username_hash`: The 32-byte username commitment.
    ///
    /// ### Returns
    /// The `PrivacyMode` for the commitment.
    pub fn get_privacy_mode(env: Env, username_hash: BytesN<32>) -> PrivacyMode {
        storage::get_privacy_mode(&env, &username_hash)
    }

    /// Resolves a commitment to a wallet address and optional memo.
    ///
    /// Returns the wallet associated with the commitment (or the contract address if shielded)
    /// along with any associated memo. The privacy mode determines what address is returned.
    ///
    /// ### Arguments
    /// - `env`: The Soroban contract environment.
    /// - `commitment`: The 32-byte username commitment.
    ///
    /// ### Returns
    /// A tuple of `(Address, Option<u64>)` where:
    /// - `Address` is the resolved wallet (or contract address if shielded).
    /// - `Option<u64>` is the associated memo, if any.
    ///
    /// ### Errors
    /// - `NotFound`: If the commitment is not registered.
    pub fn resolve(env: Env, commitment: BytesN<32>) -> (Address, Option<u64>) {
        match env
            .storage()
            .persistent()
            .get::<storage::DataKey, ResolveData>(&storage::DataKey::Resolver(commitment.clone()))
        {
            Some(data) => {
                if storage::get_privacy_mode(&env, &commitment) == PrivacyMode::Shielded {
                    (env.current_contract_address(), data.memo)
                } else {
                    (data.wallet, data.memo)
                }
            }
            None => panic_with_error!(&env, CoreError::NotFound),
        }
    }
}
