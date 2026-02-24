# Alien Gateway — Project Roadmap

This document maps the planned development of Alien Gateway across parallel workstreams. Each phase contains independent tracks that can be worked on simultaneously. Dependencies between phases are noted explicitly.

> Issues marked ✅ are already closed. Issues marked 🔲 are open and available to work on.

---

## Overview

```
Phase 1 — Foundation (DONE)
│
├── [ZK]       Workspace Setup ✅
├── [ZK]       Merkle Path Verification ✅
├── [ZK]       Merkle Inclusion Proof ✅
├── [Contract] Contract Initialization ✅
└── [Contract] Auth Middleware ✅

Phase 2 — Core Primitives (IN PROGRESS — all parallelizable)
│
├── [ZK]       Username Hash Circuit 🔲
├── [ZK]       Merkle Non-Inclusion Proof 🔲
├── [ZK]       Merkle Update Proof 🔲
├── [Contract] Add Stellar Address 🔲
└── [Contract] Add External Chain Address 🔲

Phase 3 — Off-chain Proof Layer (depends on Phase 2 ZK)
│
├── [ZK]       Username Exists Proof (Off-chain) 🔲
├── [ZK]       Proof Generation Scripts
└── [ZK]       Off-chain Verifier Integration

Phase 4 — On-chain ZK Verification (depends on Phase 3)
│
├── [Contract] ZK Verifier Contract (Groth16/PLONK)
├── [Contract] On-chain Root Anchoring
└── [Contract] Proof Submission Endpoint

Phase 5 — Resolution & Payment Layer (depends on Phase 4)
│
├── [Contract] Username → Address Resolver
├── [Contract] Stellar Memo Routing
├── [Contract] Escrow / Payment Flow
└── [SDK]      Off-chain Resolver Client

Phase 6 — Developer Experience & Hardening
│
├── [Docs]     API Reference
├── [Test]     End-to-end Test Suite
├── [CI]       GitHub Actions Pipeline
└── [Security] Audit Prep & Threat Modeling
```

---

## Phase 1 — Foundation `COMPLETE`

All foundational work is closed. These establish the ZK tooling baseline and the core Soroban contract scaffold.

| # | Track | Issue | Status |
|---|-------|-------|--------|
| [#2](https://github.com/Alien-Protocol/Alien-Gateway/issues/2) | ZK | ZK Workspace Setup (Circom + Trusted Setup) | ✅ Closed |
| [#5](https://github.com/Alien-Protocol/Alien-Gateway/issues/5) | ZK | Merkle Path Verification Circuit | ✅ Closed |
| [#6](https://github.com/Alien-Protocol/Alien-Gateway/issues/6) | ZK | Merkle Inclusion Proof Circuit | ✅ Closed |
| [#9](https://github.com/Alien-Protocol/Alien-Gateway/issues/9) | Contract | Initialize Contract (username + owner) | ✅ Closed |
| [#11](https://github.com/Alien-Protocol/Alien-Gateway/issues/11) | Contract | Set Master Stellar Address | ✅ Closed |
| [#13](https://github.com/Alien-Protocol/Alien-Gateway/issues/13) | Contract | Auth Middleware | ✅ Closed |

---

## Phase 2 — Core Primitives `IN PROGRESS`

**All issues in this phase are independent and can be worked on in parallel.**

### ZK Track

| # | Issue | Priority | Difficulty | Assignee |
|---|-------|----------|------------|---------|
| [#3](https://github.com/Alien-Protocol/Alien-Gateway/issues/3) | Username Hash Circuit (private input → public hash via Poseidon) | LOW | ☕ one-coffee | open |
| [#7](https://github.com/Alien-Protocol/Alien-Gateway/issues/7) | Merkle Non-Inclusion Proof Circuit (availability check without revealing username) | HIGH | ☕☕☕ all-nighter | open |
| [#8](https://github.com/Alien-Protocol/Alien-Gateway/issues/8) | Merkle Update Proof Circuit (prove valid state transition when a leaf is inserted) | HIGH | ☕☕☕ all-nighter | open |

### Contract Track

| # | Issue | Priority | Difficulty |
|---|-------|----------|------------|
| [#10](https://github.com/Alien-Protocol/Alien-Gateway/issues/10) | Add Stellar Address (`add_stellar_address`, dedup, auth, events) | MED | medium |
| [#12](https://github.com/Alien-Protocol/Alien-Gateway/issues/12) | Add External Chain Address (`add_chain_address` for EVM/BTC/Solana) | MED | medium |

> **Note:** #10 and #12 share `address_manager.rs` and `types.rs`. Coordinate to avoid conflicts — one contributor should handle both, or coordinate on types first.

---

## Phase 3 — Off-chain Proof Layer

**Depends on:** Phase 2 ZK track (#3, #7, #8)

These issues build the off-chain proof generation and verification layer that will later be hooked into the contract.

| # | Issue | Description | Parallelizable |
|---|-------|-------------|---------------|
| [#4](https://github.com/Alien-Protocol/Alien-Gateway/issues/4) | Username Exists in Merkle Tree (Off-chain Proof) | Full off-chain proof that a username is in the registry without revealing it | ✅ Yes |
| — | Proof Generation Scripts (`prove_non_inclusion.ts`, `prove_update.ts`) | TypeScript scripts wrapping snarkjs for each circuit type | ✅ Yes |
| — | Off-chain Verifier | Node.js verifier that validates proofs before submitting to chain | ✅ Yes |

---

## Phase 4 — On-chain ZK Verification

**Depends on:** Phase 3

Bring ZK proofs on-chain. The Soroban contract must be able to verify Groth16 proofs generated off-chain.

| # | Issue | Description | Parallelizable |
|---|-------|-------------|---------------|
| — | ZK Verifier Contract | Soroban contract implementing Groth16 or PLONK verifier for the registry circuits | With root anchoring |
| — | On-chain Root Anchoring | Store and update the canonical Merkle root on-chain after each verified update | With verifier |
| — | Proof Submission Endpoint | `submit_proof(proof, public_signals)` entry point on the contract | After verifier |

---

## Phase 5 — Resolution & Payment Layer

**Depends on:** Phase 4

The user-facing feature set: resolving usernames to addresses and routing payments.

| # | Issue | Description | Parallelizable |
|---|-------|-------------|---------------|
| — | Username → Address Resolver | `resolve(username_hash) → Address` contract function | ✅ Yes |
| — | Stellar Memo Routing | Route payments using Stellar transaction memos tied to resolved usernames | ✅ Yes |
| — | Escrow / Payment Flow | Optional escrow for payments to usernames not yet claimed | After resolver |
| — | Off-chain Resolver Client | TypeScript/JS SDK for resolving usernames and building payment transactions | ✅ Yes |

---

## Phase 6 — Developer Experience & Hardening

**Depends on:** Phase 5 (for complete API surface)
**Partially parallelizable with Phase 5.**

| # | Issue | Description | Parallelizable |
|---|-------|-------------|---------------|
| — | API Reference | Full documentation of all contract entry points and SDK methods | ✅ Yes |
| — | End-to-end Test Suite | Integration tests covering the full flow: register → prove → resolve → pay | After Phase 5 |
| — | GitHub Actions CI | Automated `cargo test`, circuit compilation, and proof verification on every PR | ✅ Yes |
| — | Audit Prep & Threat Modeling | Review ZK circuit constraints, contract auth, and Merkle tree integrity assumptions | After Phase 5 |

---

## Parallelization Map

The following groups can be worked on simultaneously right now:

```
TODAY (no blockers):
┌─────────────────────────────────┐  ┌─────────────────────────────────┐
│  ZK Track                       │  │  Contract Track                 │
│                                 │  │                                 │
│  #3  Username Hash Circuit      │  │  #10 Add Stellar Address        │
│  #7  Non-Inclusion Proof        │  │  #12 Add External Chain Address │
│  #8  Merkle Update Proof        │  │                                 │
│  #4  Username Exists (off-chain)│  │                                 │
└─────────────────────────────────┘  └─────────────────────────────────┘

AFTER PHASE 2 ZK completes:
┌─────────────────────────────────┐
│  Phase 3 — Proof Scripts        │
│                                 │
│  prove_non_inclusion.ts         │
│  prove_update.ts                │
│  Off-chain verifier             │
└─────────────────────────────────┘

AFTER PHASE 3:
┌─────────────────────────────────┐
│  Phase 4 — On-chain ZK          │
│                                 │
│  ZK Verifier Contract           │
│  Root Anchoring                 │
│  Proof Submission               │
└─────────────────────────────────┘
```

---

## Contribution

Each phase issue should be opened as a GitHub Issue following the format in [CONTRIBUTING.md](./CONTRIBUTING.md).

- Use branch prefix matching the track: `feat/zk-non-inclusion`, `feat/contract-resolver`
- Link PRs to their issue: `Closes #N`
- Issues within the same phase can be worked in parallel — coordinate on shared files (e.g., `types.rs`) via the issue thread before starting
