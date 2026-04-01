# Factory Contract Specification

The Factory contract is the username deployment registry. It maps SHA-256 username hashes to on-chain ownership records, gate-keeping deployments through the auction contract and linking every username to the core resolver contract.

---

## Function: `configure`

Stores the addresses of the auction contract and the core resolver contract. Must be called before any username can be deployed.

### Interface

```rust
pub fn configure(env: Env, auction_contract: Address, core_contract: Address)
```

### Requirements & Validation

- **Authentication**: None — any caller may invoke `configure`. Intended to be called once by the deployer during initial setup.
- **Parameters**:
  - `auction_contract` — the address of the auction contract that is authorized to trigger deployments.
  - `core_contract` — the address of the core resolver contract attached to each deployed username.

### State Changes

1. **Instance Storage**: Writes `auction_contract` to `DataKey::AuctionContract`.
2. **Instance Storage**: Writes `core_contract` to `DataKey::CoreContract`.

### Events

None.

### Security Considerations

- `configure` has no access control; it can be called repeatedly with new values. In production the deployer should call it once immediately after deployment.

---

## Function: `deploy_username`

Records a new username-to-owner mapping after the auction contract has validated the winning bid. This is the only path through which usernames enter the registry.

### Interface

```rust
pub fn deploy_username(env: Env, username_hash: BytesN<32>, owner: Address)
```

### Requirements & Validation

- **Authentication**: The **auction contract** must authorize the call (`auction_contract.require_auth()`). Any other caller is rejected.
- **Auction Contract Configured**: The auction contract address must have been set via `configure`. If not, panics with `FactoryError::Unauthorized` (code `1`).
- **Core Contract Configured**: The core contract address must have been set via `configure`. If not, panics with `FactoryError::CoreContractNotConfigured` (code `3`).
- **Uniqueness**: The `username_hash` must not already exist in persistent storage. If it does, panics with `FactoryError::AlreadyDeployed` (code `2`).

### State Changes

1. **Persistent Storage**: Creates a `UsernameRecord` at `DataKey::Username(username_hash)` containing:
   - `username_hash: BytesN<32>` — the SHA-256 hash of the username.
   - `owner: Address` — the winning bidder's address.
   - `registered_at: u64` — ledger timestamp at deployment time.
   - `core_contract: Address` — the core resolver address stored at configure time.
2. **TTL Extension**: The username entry is bumped to ~30 days (`PERSISTENT_BUMP_AMOUNT = 518_400`), with auto-extend triggered when remaining TTL drops below ~7 days (`PERSISTENT_LIFETIME_THRESHOLD = 120_960`).

### Events

| Symbol      | Topics            | Data                                                  |
|-------------|-------------------|-------------------------------------------------------|
| `USR_DEP`   | `(USR_DEP,)`      | `(username_hash: BytesN<32>, owner: Address, registered_at: u64)` |

### Errors

| Code | Variant                      | Condition                                          |
|------|------------------------------|----------------------------------------------------|
| 1    | `Unauthorized`               | Auction contract address is not configured.        |
| 2    | `AlreadyDeployed`            | `username_hash` already exists in storage.         |
| 3    | `CoreContractNotConfigured`  | Core contract address is not configured.           |

### Security Considerations

- **Reentrancy**: Not applicable — no external cross-contract calls are made during storage writes.
- **Authorization**: Hardened by host-level `require_auth` on the auction contract address, so only the auction contract can trigger deployments.
- **Double-Deploy**: Protected by the `has_username` check before writing.

---

## Function: `get_username_record`

Returns the full ownership record for a deployed username, or `None` if the username has not been registered.

### Interface

```rust
pub fn get_username_record(env: Env, username_hash: BytesN<32>) -> Option<UsernameRecord>
```

### Requirements & Validation

- **Authentication**: None — read-only, safe for public queries.

### State Changes

None — read-only.

### Events

None.

---

## Function: `get_username_owner`

Returns only the `owner` address for a deployed username, or `None` if the username has not been registered.

### Interface

```rust
pub fn get_username_owner(env: Env, username_hash: BytesN<32>) -> Option<Address>
```

### Requirements & Validation

- **Authentication**: None — read-only, safe for public polling.

### State Changes

None — read-only. **Complexity**: O(1) — single persistent storage lookup.

### Events

None.

---

## Function: `get_auction_contract`

Returns the auction contract address stored during `configure`, or `None` if not yet configured.

### Interface

```rust
pub fn get_auction_contract(env: Env) -> Option<Address>
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Function: `get_core_contract`

Returns the core resolver contract address stored during `configure`, or `None` if not yet configured.

### Interface

```rust
pub fn get_core_contract(env: Env) -> Option<Address>
```

### Requirements & Validation

- **Authentication**: None — read-only.

### State Changes

None — read-only.

### Events

None.

---

## Types

### `UsernameRecord`

```rust
pub struct UsernameRecord {
    pub username_hash: BytesN<32>,
    pub owner: Address,
    pub registered_at: u64,
    pub core_contract: Address,
}
```

### `DeployConfig` _(unused — reserved)_

```rust
pub struct DeployConfig {
    pub core_contract_wasm_hash: BytesN<32>,
    pub admin: Address,
}
```

## Storage Layout

| Key                          | Tier       | Value              | Description                          |
|------------------------------|------------|--------------------|--------------------------------------|
| `DataKey::AuctionContract`   | Instance   | `Address`          | Authorized auction contract address  |
| `DataKey::CoreContract`      | Instance   | `Address`          | Core resolver contract address       |
| `DataKey::Username(hash)`    | Persistent | `UsernameRecord`   | Per-username ownership record        |
| `DataKey::Config`            | Persistent | `DeployConfig`     | Reserved for future use              |

## Error Reference

| Code | Variant                     | Description                              |
|------|-----------------------------|------------------------------------------|
| 1    | `Unauthorized`              | Auction contract not configured          |
| 2    | `AlreadyDeployed`           | Username hash already registered         |
| 3    | `CoreContractNotConfigured` | Core contract not configured             |
