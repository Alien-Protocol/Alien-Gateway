use crate::types::{Proof, PublicSignals};
use soroban_sdk::Env;

pub struct ZkVerifier;

impl ZkVerifier {
    /// Verify a Groth16 non-inclusion proof against the given public signals.
    ///
    /// Validates structural integrity of the proof and rejects empty or trivially
    /// forged payloads. Full BN254 pairing verification will replace this in Phase 4
    /// once the on-chain ZK verifier contract is deployed.
    ///
    /// TODO(phase-4): replace with a cross-contract call to the ZK verifier once
    /// it is available on-chain.
    pub fn verify_groth16_proof(
        _env: &Env,
        proof: &Proof,
        _public_signals: &PublicSignals,
    ) -> bool {
        // Fail closed: reject empty or undersized proof payloads
        if proof.len() < 64 {
            return false;
        }

        // Reject trivially zeroed proofs
        let is_all_zero = (0..proof.len()).all(|i| proof.get(i).unwrap_or(0) == 0);
        if is_all_zero {
            return false;
        }

        true
    }
}
