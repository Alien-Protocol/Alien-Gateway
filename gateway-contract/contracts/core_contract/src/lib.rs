#![no_std]
pub mod events;
pub mod zk_verifier;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Bytes, BytesN,
    Env, Vec,
};

#[contract]
pub struct Contract;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Resolver(BytesN<32>),
}

#[contracttype]
#[derive(Clone)]
pub struct ResolveData {
    pub wallet: Address,
    pub memo: Option<u64>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResolverError {
    NotFound = 1,
    InvalidProof = 2,
    InvalidProofLength = 3,
    InvalidPublicInputCount = 4,
}

#[contractimpl]
impl Contract {
    /// Verify a Groth16 proof on-chain before accepting a username registration.
    ///
    /// * `proof`         – 256-byte serialised proof (A‖B‖C in G1/G2/G1 format).
    /// * `public_inputs` – Vec of 32-byte big-endian field elements (one per circuit input).
    ///
    /// Returns `true` on success; panics with `InvalidProof` on failure.
    pub fn verify_proof(env: Env, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> bool {
        // Length guard
        if proof.len() != 256 {
            panic_with_error!(&env, ResolverError::InvalidProofLength);
        }
        // We expect exactly 1 public input (username commitment)
        if public_inputs.len() != 1 {
            panic_with_error!(&env, ResolverError::InvalidPublicInputCount);
        }

        // Copy proof bytes into a fixed-size stack buffer
        let mut proof_buf = [0u8; 256];
        for (i, byte) in proof.iter().enumerate() {
            proof_buf[i] = byte;
        }

        // Convert public inputs
        let mut inputs: [[u8; 32]; 1] = [[0u8; 32]; 1];
        let raw = public_inputs.get(0).unwrap();
        for (j, b) in raw.iter().enumerate() {
            inputs[0][j] = b;
        }

        let ok = zk_verifier::verify_proof_bytes(&env, &proof_buf, &inputs);
        if !ok {
            panic_with_error!(&env, ResolverError::InvalidProof);
        }
        true
    }

    pub fn register_resolver(env: Env, commitment: BytesN<32>, wallet: Address, memo: Option<u64>) {
        let key = DataKey::Resolver(commitment);
        let data = ResolveData { wallet, memo };
        env.storage().persistent().set(&key, &data);
    }

    pub fn resolve(env: Env, commitment: BytesN<32>) -> ResolveData {
        let key = DataKey::Resolver(commitment);
        match env.storage().persistent().get::<DataKey, ResolveData>(&key) {
            Some(data) => data,
            None => panic_with_error!(&env, ResolverError::NotFound),
        }
    }
}

#[cfg(test)]
mod test;
