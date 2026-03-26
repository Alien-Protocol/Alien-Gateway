# Alien Gateway — Threat Model

> Prepared for external security audit. Last updated: 2026-03-24.

---

## System Overview

Alien Gateway is a username-to-address registry built on Soroban (Stellar). Users register a username commitment (Poseidon hash) on-chain. ZK circuits (Circom/Groth16) prove inclusion and valid state transitions in a Sparse Merkle Tree (SMT) without revealing the raw username.

**Components in scope:**
- ZK circuits: `username_hash`, `username_merkle`, `merkle_update`, `merkle_update_proof`, `merkle_inclusion`
- Soroban contracts: `core_contract`, `escrow_contract`, `factory_contract` (stub), `auction_contract` (stub)
- Trusted setup artifacts (Groth16 ceremony)

---

## Trust Assumptions

| Assumption | Risk if violated |
|---|---|
| Groth16 trusted setup is not compromised | Attacker can forge arbitrary proofs |
| Poseidon hash is collision-resistant | Two usernames map to the same leaf; double-registration |
| Poseidon is preimage-resistant | Username revealed from on-chain commitment |
| Soroban host correctly enforces `require_auth` | Auth bypass on all protected entry points |
| SMT root stored on-chain reflects the canonical off-chain tree | Stale or forged root accepted; invalid proofs pass |
| `pathIndices` are binary (enforced in-circuit) | Malformed path accepted; wrong leaf proven |

---

## ZK Circuit Threat Surface

### 1. Under-constrained signals

**`username_hash.circom`**
- `username[32]` inputs are field elements. No range check enforces that each element is a valid character code (e.g., 0–127 ASCII). A prover can supply arbitrary field values and produce a valid hash for a "username" that is not a real string.
- *Risk*: Two distinct byte representations of the same logical username could produce different hashes, breaking uniqueness. Mitigation must be enforced off-chain or via a separate range-check circuit.

**`merkle_update.circom` / `merkle_update_proof.circom`**
- `pathIndices[i]` binary constraint (`pathIndices[i] * (pathIndices[i] - 1) === 0`) is present in `username_merkle.circom` and `merkle_update.circom`, but `MerkleUpdateProof` delegates to `PathCalculator` which delegates to `BitSelector`. `BitSelector` enforces `s * (1 - s) === 0`. Constraint is sound.
- `usernameHash` (private input in `MerkleUpdateProof`) is not range-checked. Any field element is accepted as a "username hash". The circuit does not verify that `usernameHash` was produced by `UsernameHash`. This is an **under-constrained signal**: a prover can insert an arbitrary value as a leaf.
- *Risk*: Attacker inserts a leaf that is not a valid username hash, polluting the registry. Mitigation: compose `UsernameHash` inside `MerkleUpdateProof` and make the raw username the private input instead.

**`merkle_inclusion.circom`**
- `isValid` output is hardcoded to `1`. It does not reflect any computed constraint — it is always 1 if the proof verifies. This is not a soundness issue (the path equality constraint is the actual check), but it is misleading and should be removed or replaced with a meaningful signal.

### 2. Missing constraints

- No non-membership (non-inclusion) proof circuit exists yet (roadmap Phase 2 #7). Until it exists, the system cannot prove a username slot is empty without trusting the prover's claim that `oldLeaf === 0`.
- `MerkleUpdate` enforces `oldLeaf === 0` as a hard constraint, which is correct for insertion. However, there is no circuit preventing the same path from being used twice to insert two different leaves (replay of the same `oldRoot`). The contract must reject a root update if `oldRoot` does not match the current stored root.

### 3. Trusted setup (Groth16)

- The current setup uses a Powers of Tau ceremony. If any participant in the ceremony retained their toxic waste, they can generate fake proofs for any statement.
- The `zk/scripts/trusted-setup.sh` script performs a local setup. For production, a multi-party ceremony (e.g., Hermez, Semaphore) is required.
- Verification keys must be pinned in the contract. Any re-keying requires a contract upgrade.

---

## Contract Authorization Threat Surface

### `core_contract`

| Entry point | Auth check | Risk |
|---|---|---|
| `register_resolver` | None visible in `lib.rs` | **Critical**: anyone can register any commitment to any wallet. Needs `caller.require_auth()`. |
| `resolve` | None (read-only) | Acceptable — public resolver. |
| `Registration::register` | `caller.require_auth()` | Sound. Duplicate check prevents re-registration. |
| `SmtRoot::update_root` | `require_owner()` | Sound. Owner-only. |

**Finding**: `register_resolver` in `core_contract/src/lib.rs` has no authentication. Any account can overwrite or create resolver entries for arbitrary commitments. This is a **critical auth bypass**.

### `escrow_contract`

| Entry point | Auth check | Risk |
|---|---|---|
| `schedule_payment` | `vault.owner.require_auth()` | Sound. |
| Payment execution | Not yet implemented | Future surface — must enforce `release_at <= now` and `executed == false`. |

**Finding**: There is no `execute_payment` function yet. When implemented, it must atomically check `executed`, set it to `true`, and transfer funds before any external call to prevent reentrancy and double-execution.

**Finding**: The `to` vault is not validated to exist at scheduling time. A payment can be scheduled to a non-existent vault. Define whether this is intentional (lazy creation) or a bug.

### `factory_contract` / `auction_contract`

Both are empty stubs. No threat surface yet. Flag for audit when implemented.

---

## Merkle Tree Integrity

- The on-chain root (`SmtRoot`) is updated by the owner without any ZK proof verification (Phase 4 not yet implemented). The owner is fully trusted to submit correct roots.
- Until on-chain proof verification is live, the system's integrity guarantee is: "the owner claims this root is correct." This is a **centralization risk**.
- The SMT uses Poseidon for internal nodes and leaves. Poseidon is ZK-friendly and considered collision-resistant for its parameter sets, but has not received the same volume of cryptanalysis as SHA-2/SHA-3.
- Tree depth is 20 in production circuits (`MerkleInclusionProof`, `MerkleUpdateProof`), supporting up to 2^20 (~1M) leaves. Depth 2 is used in `username_merkle.circom` and `merkle_update.circom` — these appear to be development/test instances.

---

## Summary of Findings

| ID | Severity | Location | Description |
|---|---|---|---|
| F-01 | Critical | `core_contract/src/lib.rs` | `register_resolver` has no auth check |
| F-02 | High | `merkle_update_proof.circom` | `usernameHash` private input not constrained to be a valid `UsernameHash` output |
| F-03 | High | All circuits | `username[32]` inputs not range-checked to valid character values |
| F-04 | Medium | `merkle_update.circom` | No circuit-level replay protection for same `oldRoot` |
| F-05 | Medium | `smt_root.rs` | Root updated without on-chain proof verification (owner-trusted) |
| F-06 | Medium | `escrow_contract` | `execute_payment` not implemented; double-execution risk when added |
| F-07 | Low | `merkle_inclusion.circom` | `isValid` hardcoded to 1; misleading signal |
| F-08 | Low | Trusted setup | Local ceremony only; not suitable for production |
| F-09 | Info | `escrow_contract` | `to` vault existence not validated at scheduling time |
