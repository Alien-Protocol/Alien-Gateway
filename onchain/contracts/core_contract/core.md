# Core Contract Specification

The Core contract is the central identity resolver for the Alien Gateway protocol. It manages username registrations (commitment-based), multi-chain address linking, privacy modes, ZK-verified resolver registrations, and ownership transfers. All identity state is anchored to SHA-256 commitment hashes.

---

## Function: `initialize`

Sets the contract owner address. Must be called exactly once after deployment.

### Interface

```rust
pub fn initialize(env: Env, owner: Address)
```

### Requirements & Validation

- **Authentication**: `owner.require_auth()` — the supplied address must authorize the call.
- **Idempotency**: Panics with `CoreError::AlreadyInitialized` (code `9`) if the contract has already been initialized.

### State Changes

1. **Instance Storage**: Writes `owner` to `DataKey::Owner`.

### Events

| Symbol | Topics       | Data              |
|--------|--------------|-------------------|
| `INIT` | `(INIT,)`    | `(owner: Address)` |

### Errors

| Code | Variant              | Condition                                  |
|------|----------------------|--------------------------------------------|
| 9    | `AlreadyInitialized` | `initialize` has already been called.      |

---

## Function: `get_contract_owner`

Returns the owner address set during `initialize`.

### Interface

```rust
pub fn get_contract_owner(env: Env) -> Address
```

### Requirements & Validation

- **Authentication**: None — read-only.
- Panics with `CoreError::NotFound` (code `1`) if `initialize` has not been called.

### State Changes

None — read-only.

### Events

None.

---

## Function: `register`

Registers a username commitment (Poseidon hash of the username) and maps it to the caller's wallet address. This is the simple registration path (no ZK proof required).

### Interface

```rust
pub fn register(env: Env, caller: Address, commitment: BytesN<32>)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Uniqueness**: Panics with `CoreError::AlreadyRegistered` (code `10`) if the commitment already exists.

### State Changes

1. **Persistent Storage**: Maps `Commitment(commitment)` → `caller: Address`.
2. **TTL Extension**: Entry bumped to ~30 days (`PERSISTENT_BUMP_AMOUNT = 518_400`), auto-extend at ~7 days (`PERSISTENT_LIFETIME_THRESHOLD = 120_960`).

### Events

| Symbol     | Topics           | Data                                        |
|------------|------------------|---------------------------------------------|
| `REGISTER` | `(REGISTER,)`   | `(commitment: BytesN<32>, caller: Address)` |

### Errors

| Code | Variant             | Condition                        |
|------|---------------------|----------------------------------|
| 10   | `AlreadyRegistered` | Commitment already exists.       |

---

## Function: `get_owner`

Returns the registered owner of a commitment, or `None` if not registered.

### Interface

```rust
pub fn get_owner(env: Env, commitment: BytesN<32>) -> Option<Address>
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Function: `register_resolver`

Registers a ZK-verified resolver entry. Validates a Groth16 non-inclusion proof against the current SMT root, stores the resolver data, and advances the SMT root.

### Interface

```rust
pub fn register_resolver(
    env: Env,
    caller: Address,
    commitment: BytesN<32>,
    proof: Bytes,
    public_signals: PublicSignals,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Uniqueness**: Panics with `CoreError::DuplicateCommitment` (code `3`) if the commitment already exists as a resolver entry.
- **SMT Root**: `public_signals.old_root` must equal the current on-chain SMT root. Panics with `CoreError::RootNotSet` (code `2`) if no root exists, or `CoreError::StaleRoot` (code `4`) on mismatch.
- **Proof**: The Groth16 proof must pass verification. Panics with `CoreError::InvalidProof` (code `5`) on failure. Proof must be ≥ 64 bytes and non-zero.

### State Changes

1. **Persistent Storage**: Creates `ResolveData { wallet: caller, memo: None }` at `DataKey::Resolver(commitment)`.
2. **TTL Extension**: Resolver entry bumped to ~30 days.
3. **SMT Root Update**: Instance storage `DataKey::SmtRoot` is updated to `public_signals.new_root`.

### Events

| Symbol       | Topics           | Data                                          |
|--------------|------------------|-----------------------------------------------|
| `REGISTER`   | `(REGISTER,)`   | `(commitment: BytesN<32>, caller: Address)`   |
| `ROOT_UPD`   | `(ROOT_UPD,)`   | `(old_root: Option<BytesN<32>>, new_root: BytesN<32>)` |

### Errors

| Code | Variant              | Condition                                    |
|------|----------------------|----------------------------------------------|
| 2    | `RootNotSet`         | SMT root has not been initialized.           |
| 3    | `DuplicateCommitment`| Commitment already registered as resolver.   |
| 4    | `StaleRoot`          | `old_root` does not match on-chain root.     |
| 5    | `InvalidProof`       | Groth16 proof failed verification.           |

### Security Considerations

- **ZK Verification**: Currently uses structural validation (≥64 bytes, non-zero). Full BN254 pairing verification is planned for Phase 4.
- **Root Consistency**: Old root check prevents replay of stale proofs.

---

## Function: `resolve`

Resolves a commitment to its linked wallet address and optional memo. Respects privacy mode: if the commitment is set to `Shielded`, returns the contract's own address instead of the wallet.

### Interface

```rust
pub fn resolve(env: Env, commitment: BytesN<32>) -> (Address, Option<u64>)
```

### Requirements & Validation

- **Authentication**: None — read-only.
- Panics with `CoreError::NotFound` (code `1`) if no resolver data exists for the commitment.

### State Changes

None — read-only.

### Events

None.

### Implementation Details

- If `PrivacyMode::Shielded`, returns `(env.current_contract_address(), memo)` — the real wallet is never exposed.
- If `PrivacyMode::Normal` (default), returns `(wallet, memo)`.

---

## Function: `set_memo`

Sets or updates the memo field on an existing resolver entry.

### Interface

```rust
pub fn set_memo(env: Env, commitment: BytesN<32>, memo_id: u64)
```

### Requirements & Validation

- **Authentication**: None (the resolver entry must already exist).
- Panics with `CoreError::NotFound` (code `1`) if the commitment has no resolver data.

### State Changes

1. **Persistent Storage**: Updates `ResolveData.memo` to `Some(memo_id)` at `DataKey::Resolver(commitment)`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

None.

---

## Function: `set_privacy_mode`

Sets the privacy mode for a username hash. Only the registered owner may change the mode.

### Interface

```rust
pub fn set_privacy_mode(env: Env, username_hash: BytesN<32>, mode: PrivacyMode)
```

### Requirements & Validation

- **Authentication**: The registered owner of `username_hash` must authorize the call (`owner.require_auth()`).
- Panics with `CoreError::NotFound` (code `1`) if the username hash is not registered.

### State Changes

1. **Persistent Storage**: Writes `mode` to `DataKey::PrivacyMode(username_hash)`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

| Symbol         | Topics              | Data                                           |
|----------------|---------------------|-------------------------------------------------|
| `PRIVACY_SET`  | `(PRIVACY_SET,)`   | `(username_hash: BytesN<32>, mode: PrivacyMode)` |

---

## Function: `get_privacy_mode`

Returns the privacy mode for a username hash. Defaults to `PrivacyMode::Normal` if not explicitly set.

### Interface

```rust
pub fn get_privacy_mode(env: Env, username_hash: BytesN<32>) -> PrivacyMode
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Function: `get_smt_root`

Returns the current Sparse Merkle Tree root.

### Interface

```rust
pub fn get_smt_root(env: Env) -> BytesN<32>
```

### Requirements & Validation

- **Authentication**: None — read-only.
- Panics with `CoreError::RootNotSet` (code `2`) if no root has been set.

### State Changes

None — read-only.

### Events

None.

---

## Function: `transfer_ownership`

Transfers ownership of a commitment to a new address. Simple transfer path (no ZK proof).

### Interface

```rust
pub fn transfer_ownership(
    env: Env,
    caller: Address,
    commitment: BytesN<32>,
    new_owner: Address,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the current registered owner. Panics with `CoreError::NotFound` (code `1`) if the commitment does not exist, or `CoreError::Unauthorized` (code `7`) if the caller is not the owner.
- **Distinct Owner**: `new_owner` must differ from the current owner. Panics with `CoreError::SameOwner` (code `8`) otherwise.

### State Changes

1. **Persistent Storage**: Updates `Commitment(commitment)` → `new_owner`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

| Symbol     | Topics           | Data                                                            |
|------------|------------------|-----------------------------------------------------------------|
| `TRANSFER` | `(TRANSFER,)`   | `(commitment: BytesN<32>, old_owner: Address, new_owner: Address)` |

### Errors

| Code | Variant        | Condition                                |
|------|----------------|------------------------------------------|
| 1    | `NotFound`     | Commitment does not exist.               |
| 7    | `Unauthorized` | Caller is not the registered owner.      |
| 8    | `SameOwner`    | `new_owner` equals current owner.        |

---

## Function: `transfer`

Transfers ownership of a commitment with ZK proof verification and SMT root update. This is the privacy-preserving transfer path.

### Interface

```rust
pub fn transfer(
    env: Env,
    caller: Address,
    commitment: BytesN<32>,
    new_owner: Address,
    proof: Bytes,
    public_signals: PublicSignals,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the current registered owner. Panics with `CoreError::NotFound` (code `1`) or `CoreError::Unauthorized` (code `7`).
- **Distinct Owner**: Panics with `CoreError::SameOwner` (code `8`) if `new_owner` equals the current owner.
- **SMT Root**: `public_signals.old_root` must match the current on-chain root. Panics with `CoreError::RootNotSet` (code `2`) or `CoreError::StaleRoot` (code `4`).
- **Proof**: Groth16 proof must pass verification. Panics with `CoreError::InvalidProof` (code `5`).

### State Changes

1. **Persistent Storage**: Updates `Commitment(commitment)` → `new_owner`.
2. **TTL Extension**: Entry bumped to ~30 days.
3. **SMT Root Update**: Instance storage `DataKey::SmtRoot` updated to `public_signals.new_root`.

### Events

| Symbol       | Topics           | Data                                                              |
|--------------|------------------|-------------------------------------------------------------------|
| `TRANSFER`   | `(TRANSFER,)`   | `(commitment: BytesN<32>, old_owner: Address, new_owner: Address)` |
| `ROOT_UPD`   | `(ROOT_UPD,)`   | `(old_root: Option<BytesN<32>>, new_root: BytesN<32>)`           |

### Errors

| Code | Variant        | Condition                                    |
|------|----------------|----------------------------------------------|
| 1    | `NotFound`     | Commitment does not exist.                   |
| 2    | `RootNotSet`   | SMT root has not been set.                   |
| 4    | `StaleRoot`    | `old_root` does not match on-chain root.     |
| 5    | `InvalidProof` | Groth16 proof failed verification.           |
| 7    | `Unauthorized` | Caller is not the registered owner.          |
| 8    | `SameOwner`    | `new_owner` equals current owner.            |

---

## Function: `add_chain_address`

Links a cross-chain address (EVM, Bitcoin, Solana, Cosmos) to a registered username hash. Only the registered owner may add addresses.

### Interface

```rust
pub fn add_chain_address(
    env: Env,
    caller: Address,
    username_hash: BytesN<32>,
    chain: ChainType,
    address: Bytes,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the registered owner of `username_hash`. Panics with `ChainAddressError::NotRegistered` (code `2`) if the commitment is not registered, or `ChainAddressError::Unauthorized` (code `1`) if the caller is not the owner.
- **Address Format**: The address must pass chain-specific validation:
  - `Evm` — exactly 42 bytes, starts with `0x`.
  - `Bitcoin` — 25–62 bytes.
  - `Solana` — 32–44 bytes.
  - `Cosmos` — 39–45 bytes.
  - Panics with `ChainAddressError::InvalidAddress` (code `3`) on invalid format.

### State Changes

1. **Persistent Storage**: Writes `address` to `ChainAddrKey::ChainAddress(username_hash, chain)`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

| Symbol      | Topics           | Data                                                           |
|-------------|------------------|----------------------------------------------------------------|
| `CHAIN_ADD` | `(CHAIN_ADD,)`  | `(username_hash: BytesN<32>, chain: ChainType, address: Bytes)` |

### Errors

| Code | Variant          | Condition                               |
|------|------------------|-----------------------------------------|
| 1    | `Unauthorized`   | Caller is not the owner.                |
| 2    | `NotRegistered`  | Username commitment is not registered.  |
| 3    | `InvalidAddress` | Address format invalid for chain type.  |

---

## Function: `get_chain_address`

Returns the linked address for a username hash and chain type, or `None` if not set.

### Interface

```rust
pub fn get_chain_address(
    env: Env,
    username_hash: BytesN<32>,
    chain: ChainType,
) -> Option<Bytes>
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Function: `remove_chain_address`

Removes a cross-chain address link for a registered username hash. Only the registered owner may remove addresses.

### Interface

```rust
pub fn remove_chain_address(
    env: Env,
    caller: Address,
    username_hash: BytesN<32>,
    chain: ChainType,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the registered owner. Panics with `ChainAddressError::NotRegistered` (code `2`) or `ChainAddressError::Unauthorized` (code `1`).

### State Changes

1. **Persistent Storage**: Removes the entry at `ChainAddrKey::ChainAddress(username_hash, chain)`.

### Events

| Symbol      | Topics           | Data                                            |
|-------------|------------------|-------------------------------------------------|
| `CHAIN_REM` | `(CHAIN_REM,)`  | `(username_hash: BytesN<32>, chain: ChainType)` |

### Errors

| Code | Variant          | Condition                              |
|------|------------------|----------------------------------------|
| 1    | `Unauthorized`   | Caller is not the owner.               |
| 2    | `NotRegistered`  | Username commitment is not registered. |

---

## Function: `add_stellar_address`

Links a primary Stellar address to a registered username hash.

### Interface

```rust
pub fn add_stellar_address(
    env: Env,
    caller: Address,
    username_hash: BytesN<32>,
    stellar_address: Address,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the registered owner of `username_hash`. Panics with `CoreError::NotFound` (code `1`) if the username is not registered or the caller is not the owner.

### State Changes

1. **Persistent Storage**: Writes `stellar_address` to `DataKey::StellarAddress(username_hash)`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

None.

---

## Function: `resolve_stellar`

Resolves a username hash to its primary linked Stellar address.

### Interface

```rust
pub fn resolve_stellar(env: Env, username_hash: BytesN<32>) -> Address
```

### Requirements & Validation

- **Authentication**: None — read-only.
- Panics with `CoreError::NotFound` (code `1`) if the username hash is not registered.
- Panics with `CoreError::NoAddressLinked` (code `6`) if the username is registered but has no primary Stellar address.

### State Changes

None — read-only.

### Events

None.

### Errors

| Code | Variant           | Condition                                     |
|------|-------------------|-----------------------------------------------|
| 1    | `NotFound`        | Username hash is not registered.              |
| 6    | `NoAddressLinked` | Registered but no Stellar address is linked.  |

---

## Function: `add_shielded_address`

Stores a ZK commitment as the shielded address for a username. The raw address is never stored on-chain — only the commitment (ZK proof handle).

### Interface

```rust
pub fn add_shielded_address(
    env: Env,
    caller: Address,
    username_hash: BytesN<32>,
    address_commitment: BytesN<32>,
)
```

### Requirements & Validation

- **Authentication**: `caller.require_auth()`.
- **Ownership**: Caller must be the registered owner. Panics with `CoreError::NotFound` (code `1`) if not registered, or `CoreError::Unauthorized` (code `7`) if the caller is not the owner.

### State Changes

1. **Persistent Storage**: Writes `address_commitment` to `DataKey::ShieldedAddress(username_hash)`.
2. **TTL Extension**: Entry bumped to ~30 days.

### Events

| Symbol         | Topics              | Data                                                            |
|----------------|---------------------|-----------------------------------------------------------------|
| `SHIELDED_ADD` | `(SHIELDED_ADD,)`  | `(username_hash: BytesN<32>, address_commitment: BytesN<32>)`   |

---

## Function: `get_shielded_address`

Returns the shielded address commitment for a username hash, or `None` if not set.

### Interface

```rust
pub fn get_shielded_address(env: Env, username_hash: BytesN<32>) -> Option<BytesN<32>>
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Function: `is_shielded`

Returns `true` if a shielded address commitment has been stored for the given username hash.

### Interface

```rust
pub fn is_shielded(env: Env, username_hash: BytesN<32>) -> bool
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Types

### `ResolveData`

```rust
pub struct ResolveData {
    pub wallet: Address,
    pub memo: Option<u64>,
}
```

### `ChainType`

```rust
pub enum ChainType {
    Evm,
    Bitcoin,
    Solana,
    Cosmos,
}
```

### `PrivacyMode`

```rust
pub enum PrivacyMode {
    Normal,
    Shielded,
}
```

### `PublicSignals`

```rust
pub struct PublicSignals {
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
}
```

## Storage Layout

| Key                                    | Tier       | Value           | Description                                     |
|----------------------------------------|------------|-----------------|-------------------------------------------------|
| `DataKey::Owner`                       | Instance   | `Address`       | Contract owner set during `initialize`          |
| `DataKey::SmtRoot`                     | Instance   | `BytesN<32>`    | Current Sparse Merkle Tree root                 |
| `DataKey::Resolver(commitment)`        | Persistent | `ResolveData`   | ZK-verified resolver entry                      |
| `DataKey::StellarAddress(hash)`        | Persistent | `Address`       | Primary Stellar address for a username          |
| `DataKey::PrivacyMode(hash)`           | Persistent | `PrivacyMode`   | Per-username privacy setting                    |
| `DataKey::ShieldedAddress(hash)`       | Persistent | `BytesN<32>`    | ZK commitment for shielded address              |
| `Commitment(commitment)`               | Persistent | `Address`       | Username registration → owner mapping           |
| `ChainAddrKey::ChainAddress(hash, chain)` | Persistent | `Bytes`      | Cross-chain address (EVM, BTC, SOL, ATOM)       |

## Error Reference

### `CoreError`

| Code | Variant              | Description                                          |
|------|----------------------|------------------------------------------------------|
| 1    | `NotFound`           | Requested resource does not exist.                   |
| 2    | `RootNotSet`         | SMT root has not been initialized.                   |
| 3    | `DuplicateCommitment`| Commitment already registered as a resolver.         |
| 4    | `StaleRoot`          | Supplied `old_root` does not match on-chain root.    |
| 5    | `InvalidProof`       | Groth16 proof failed verification.                   |
| 6    | `NoAddressLinked`    | Username registered but no Stellar address linked.   |
| 7    | `Unauthorized`       | Caller is not the registered owner.                  |
| 8    | `SameOwner`          | Transfer target is the same as the current owner.    |
| 9    | `AlreadyInitialized` | `initialize()` has already been called.              |
| 10   | `AlreadyRegistered`  | Commitment already registered via `register()`.      |

### `ChainAddressError`

| Code | Variant          | Description                                  |
|------|------------------|----------------------------------------------|
| 1    | `Unauthorized`   | Caller is not the owner of the commitment.   |
| 2    | `NotRegistered`  | Username commitment is not registered.       |
| 3    | `InvalidAddress` | Address format is invalid for the chain type.|
