# Contributing to Alien Gateway

Thank you for your interest in contributing to Alien Gateway — a privacy-preserving username and payment resolution layer built on Stellar.

This guide covers everything you need to get started: setting up your environment, branching strategy, commit standards, and the PR process.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Forking & Cloning](#forking--cloning)
- [Project Structure](#project-structure)
- [Branching Strategy](#branching-strategy)
- [Making Changes](#making-changes)
- [Running Tests](#running-tests)
- [Commit Standards](#commit-standards)
- [Pull Request Process](#pull-request-process)
- [Syncing Your Fork](#syncing-your-fork)
- [Best Practices](#best-practices)

---

## Prerequisites

Before contributing, make sure you have the following installed:

| Tool | Purpose | Version |
|------|---------|---------|
| [Rust](https://www.rust-lang.org/tools/install) | Soroban smart contracts | `>=1.78` (stable) |
| [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli) | Deploy and interact with Soroban contracts | latest |
| [Node.js](https://nodejs.org/) | ZK circuit tooling | `>=18` |
| [Circom](https://docs.circom.io/getting-started/installation/) | ZK circuit compilation | `>=2.1` |
| [snarkjs](https://github.com/iden3/snarkjs) | Proof generation and verification | `>=0.7` |

### Install Rust targets for Soroban

```bash
rustup target add wasm32-unknown-unknown
```

### Install Stellar CLI

```bash
cargo install --locked stellar-cli --features opt
```

### Install ZK dependencies

```bash
cd zk
npm install
```

---

## Forking & Cloning

1. **Fork** the repository on GitHub via the "Fork" button on the top right.

2. **Clone** your fork locally:

```bash
git clone https://github.com/<your-username>/Alien-Gateway.git
cd Alien-Gateway
```

3. **Add the upstream remote** to keep your fork in sync:

```bash
git remote add upstream https://github.com/Alien-Protocol/Alien-Gateway.git
```

4. Verify your remotes:

```bash
git remote -v
# origin    https://github.com/<your-username>/Alien-Gateway.git (fetch)
# origin    https://github.com/<your-username>/Alien-Gateway.git (push)
# upstream  https://github.com/Alien-Protocol/Alien-Gateway.git (fetch)
# upstream  https://github.com/Alien-Protocol/Alien-Gateway.git (push)
```

---

## Project Structure

```
Alien-Gateway/
├── gateway-contract/          # Soroban smart contracts (Rust)
│   ├── contracts/
│   │   └── alien-gateway/
│   │       └── src/
│   │           ├── lib.rs             # Entry point
│   │           ├── contract_core.rs   # Core contract logic
│   │           └── address_manager.rs # Username → address mapping
│   ├── tests/
│   │   └── integration/       # Integration tests
│   └── Cargo.toml
│
└── zk/                        # Zero-knowledge circuits (Circom)
    ├── circuits/
    │   ├── merkle/            # Merkle inclusion & path circuits
    │   ├── username_hash.circom
    │   └── hello.circom
    ├── scripts/               # Compile & trusted setup scripts
    └── package.json
```

---

## Branching Strategy

Always create a new branch from an up-to-date `main`. Use the following prefixes to keep branches organized:

| Prefix | Use for |
|--------|---------|
| `feat/` | New features |
| `fix/` | Bug fixes |
| `docs/` | Documentation changes |
| `refactor/` | Code refactoring (no behavior change) |
| `test/` | Adding or improving tests |
| `chore/` | Tooling, CI, dependency updates |

**Examples:**

```bash
git checkout -b feat/merkle-root-anchoring
git checkout -b fix/address-lookup-panic
git checkout -b docs/contributing-guide
```

Keep branch names lowercase, hyphen-separated, and descriptive.

---

## Making Changes

1. Sync your fork before starting (see [Syncing Your Fork](#syncing-your-fork)).
2. Create a branch following the naming conventions above.
3. Make focused, atomic changes — one feature or fix per branch.
4. Test your changes locally before committing (see [Running Tests](#running-tests)).

---

## Running Tests

### Soroban Contracts

Run the full test suite from the `gateway-contract` directory:

```bash
cd gateway-contract
cargo test
```

Run only integration tests:

```bash
cargo test --test integration
```

Build the contract WASM (used for deployment verification):

```bash
cargo build --target wasm32-unknown-unknown --release
```

### ZK Circuits

Compile a circuit:

```bash
cd zk
# On Unix/macOS:
bash scripts/compile.sh

# On Windows:
scripts/compile.cmd
```

Run the trusted setup:

```bash
# On Unix/macOS:
bash scripts/trusted-setup.sh

# On Windows:
scripts/trusted-setup.cmd
```

---

## Commit Standards

This project follows the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

### Types

| Type | When to use |
|------|------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or correcting tests |
| `chore` | Build process, dependency updates, tooling |
| `perf` | Performance improvement |

### Examples

```bash
feat(contract): add merkle root anchoring to registry

fix(address-manager): resolve panic on empty username lookup

docs: add contributing guide for new contributors

test(integration): add tests for username collision handling

refactor(zk): simplify poseidon hasher circuit inputs
```

**Rules:**
- Use the imperative mood in the subject line ("add" not "added" or "adds")
- Limit the subject line to 72 characters
- Do not end the subject line with a period
- Reference related issues in the footer: `Closes #21`

---

## Pull Request Process

1. Push your branch to your fork:

```bash
git push origin feat/your-feature-name
```

2. Open a PR against `Alien-Protocol/Alien-Gateway`'s `main` branch on GitHub.

3. **Link the PR to its issue** using a closing keyword in the PR description:

```
Closes #21
```

4. Fill out the PR description with:
   - **What** changed and **why**
   - Any relevant context or trade-offs
   - Steps to test or verify the change

5. Ensure all checks pass before requesting a review.

6. Be responsive to reviewer feedback — address comments and push updates to the same branch.

7. Do not force-push after a review has started unless asked.

**A PR will be merged when:**
- It passes CI checks
- It has at least one maintainer approval
- All reviewer comments are resolved

---

## Syncing Your Fork

Keep your fork up to date with upstream before starting any new work:

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

If you have a feature branch in progress:

```bash
git checkout feat/your-feature-name
git rebase main
```

Resolve any conflicts, then continue your work.

---

## Best Practices

- **One PR = one thing.** Don't mix unrelated changes in the same pull request.
- **Write clear PR descriptions.** Explain the problem, your solution, and how to verify it.
- **Keep commits clean.** Squash WIP commits before opening a PR if they don't add meaningful history.
- **Test before pushing.** Run `cargo test` for contracts and verify ZK circuits compile before submitting.
- **Ask questions early.** If you're unsure about scope or approach, open a draft PR or comment on the issue before investing significant time.
- **Respect existing conventions.** Match the code style, module organization, and naming patterns already in the codebase.
- **Security-first.** This project handles financial transactions and ZK proofs. Be especially careful with input validation, arithmetic, and any on-chain state mutation.

---

If you have questions not covered here, open a [GitHub Discussion](https://github.com/Alien-Protocol/Alien-Gateway/discussions) or comment on the relevant issue.

Happy building! 🚀
