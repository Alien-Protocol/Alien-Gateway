# Security Note: Phase 4 Groth16 Verifier Stub

## Current Status: Phase 4 Stub

The current implementation of `ZkVerifier::verify_groth16_proof` in `zk_verifier.rs` is a **Phase 4 stub**. It does not perform full BN254 pairing verification.

### Mock Behavior
The stub currently only validates:
1.  **Payload Length**: The proof bytes must be at least 64 bytes long.
2.  **Non-zero Content**: The proof MUST NOT be entirely composed of zeroed bytes.

### Security Limitations
> [!WARNING]
> **The current ZK system provides NO cryptographic guarantees in production.**
> Any non-empty, non-zero payload will be accepted as a valid proof of non-inclusion or identity.

## Path to Production (Mainnet)

Before mainnet deployment, the following must be completed to ensure the security of the Alien Gateway:

1.  **On-Chain Verifier Deployment**: Replace the stub with a cross-contract call to a dedicated BN254 Groth16 verifier contract on the Stellar network.
2.  **Trusted Ceremony**: A multi-party trusted ceremony (MPC) must be conducted to generate the production parameters (`zkey`) and the corresponding on-chain verification key.
3.  **Audit**: A full security audit of the verifier implementation and circuit logic.

## Tracking
This replacement requirement is tracked via `TODO(phase-4)` comments in the codebase. The pre-commit hooks have been configured to warn about these stubs.
