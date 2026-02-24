
# 🌉 Alien Gateway

[![Smart Contracts CI](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/build_test.yml/badge.svg)](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/build_test.yml)
[![Checks](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/checks.yml/badge.svg)](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/checks.yml)
[![ZK Circuits CI](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/zk_circuits.yml/badge.svg)](https://github.com/Alien-Protocol/Alien-Gateway/actions/workflows/zk_circuits.yml)

> **Send crypto to `@username` on Stellar**

Alien Gateway is a **Stellar Naming Service ** — a decentralized, privacy-preserving username system for crypto payments on **Stellar**.

---

## What It Does

* Maps `@username → wallet address`
* Enables payments using human-readable names
* Works natively with Stellar payments and memos
* Prevents wrong-address transfers
* Preserves user privacy using ZK proofs

---

## Why gateway

* Wallet addresses are long and error-prone
* Users manage multiple wallets across chains
* Public name registries leak identity data

SNS treats **usernames as a core payment primitive**, not just metadata.

---

## How It Works 

1. User registers `@username`
2. Username is stored as a **ZK commitment**, not plaintext
3. Availability is proven using **Circom ZK proofs**
4. Username resolves to a wallet at payment time
5. Stellar transaction is routed with memo support

---

## Zero-Knowledge Design

* Usernames are **never stored on-chain in plaintext**
* Existence / non-existence is proven via ZK
* Registry is backed by a Sparse Merkle Tree
* Only a single Merkle root is anchored on-chain

---

## Core Components

* **username Registry (Soroban)**

  * Stores commitments
  * Anchors Merkle root

* **ZK Verifier**

  * Verifies Circom proofs

* **Wallet Resolver**

  * Maps username → wallet
  * Supports exchange memos

---

## Use Cases

* Wallet payments
* Exchange deposits
* DAO payouts
* Creator tipping
* Cross-border transfers

---

## Vision

**One username. One identity. Stellar-native.**

Alien Gateway aims to be the **identity and payment resolution layer** for the Stellar ecosystem.

---

## Development

### Building the Smart Contract

```bash
cd gateway-contract
stellar contract build
```

### Running Contract Tests

```bash
cd gateway-contract
cargo test
```

### Compiling ZK Circuits

```bash
cd zk
npm install

# Compile all circuits
npm run compile:all

# Or compile individual circuits
npm run compile:hello
npm run compile:username_hash
npm run compile:merkle_path
npm run compile:merkle_inclusion
npm run compile:merkle_update
npm run compile:username_merkle
```

### CI/CD Pipeline

The project uses GitHub Actions for continuous integration:

- **Smart Contracts CI**: Builds the Soroban contract to WASM and runs all Rust tests
- **Checks**: Runs code formatting (rustfmt) and linting (clippy) checks
- **ZK Circuits CI**: Compiles all Circom circuits to ensure they build without errors

All checks must pass before PRs can be merged.

---

