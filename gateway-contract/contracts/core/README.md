# Chain Registry Contract

A Soroban smart contract for managing and storing addresses from external chains (EVM, Bitcoin, Solana, etc.).

## Features

- **Multi-chain Support**: Store addresses from multiple blockchain networks (Ethereum, Bitcoin, Solana, Polygon, Arbitrum, Optimism, Base)
- **Duplicate Prevention**: Automatically prevents duplicate addresses on the same chain
- **Owner Authorization**: Only the contract owner can add or remove chain addresses
- **Event Emission**: Emits events for all state changes
- **Address Query**: Retrieve stored addresses by chain or index

## API Reference

### Core Functions

#### `initialize(env: Env, owner: Address)`
Initialize the contract with an owner address. Must be called once before using other functions.

```rust
ChainRegistry::initialize(&env, owner);
```

#### `add_chain_address(env: Env, chain: ChainId, address: String, label: String) -> Result<(), String>`
Add a new address for a specific chain.

**Parameters:**
- `chain`: The blockchain network (Ethereum, Bitcoin, Solana, etc.)
- `address`: The address on that chain
- `label`: A human-readable label for the address

**Errors:**
- Returns error if caller is not the owner
- Returns error if address already exists for this chain

**Events:** Emits `ChainAddressAdded` event on success

```rust
let result = ChainRegistry::add_chain_address(
    &env,
    ChainId::Ethereum,
    String::from_str(&env, "0x1234567890123456789012345678901234567890"),
    String::from_str(&env, "My Ethereum Wallet"),
);
```

#### `get_chain_addresses(env: Env, chain: ChainId) -> Vec<ChainAddress>`
Retrieve all addresses for a specific chain.

```rust
let addresses = ChainRegistry::get_chain_addresses(&env, ChainId::Ethereum);
```

#### `get_chain_address_at(env: Env, chain: ChainId, index: u32) -> Result<ChainAddress, String>`
Retrieve a specific address by index for a chain.

```rust
let address = ChainRegistry::get_chain_address_at(&env, ChainId::Ethereum, 0)?;
```

#### `get_chain_address_count(env: Env, chain: ChainId) -> u32`
Get the count of addresses stored for a chain.

```rust
let count = ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum);
```

#### `has_chain_address(env: Env, chain: ChainId, address: String) -> bool`
Check if an address exists for a specific chain.

```rust
let exists = ChainRegistry::has_chain_address(
    &env,
    ChainId::Ethereum,
    String::from_str(&env, "0x1234567890123456789012345678901234567890"),
);
```

#### `remove_chain_address(env: Env, chain: ChainId, address: String) -> Result<(), String>`
Remove a chain address (owner only).

**Events:** Emits `ChainAddressRemoved` event on success

```rust
let result = ChainRegistry::remove_chain_address(
    &env,
    ChainId::Ethereum,
    String::from_str(&env, "0x1234567890123456789012345678901234567890"),
);
```

#### `get_owner(env: Env) -> Result<Address, String>`
Get the current contract owner.

### Owner Management

#### `change_owner(env: Env, new_owner: Address) -> Result<(), String>`
Change the contract owner (current owner only).

**Events:** Emits `OwnerChanged` event on success

## Supported Chains

The contract supports the following blockchain networks via the `ChainId` enum:

- `Ethereum` - Ethereum mainnet
- `Bitcoin` - Bitcoin network
- `Solana` - Solana blockchain
- `Polygon` - Polygon (formerly Matic)
- `Arbitrum` - Arbitrum One
- `Optimism` - Optimism mainnet
- `Base` - Base network

## Data Types

### ChainAddress
```rust
pub struct ChainAddress {
    pub chain: ChainId,
    pub address: String,
    pub label: String,
}
```

### ChainRegistryEvent
Events that can be emitted:
- `ChainAddressAdded { chain, address, label }` - Emitted when a new address is added
- `ChainAddressRemoved { chain, address }` - Emitted when an address is removed
- `OwnerChanged { new_owner }` - Emitted when the owner is changed

## Testing

Comprehensive tests cover:
- ✅ Adding addresses per chain
- ✅ Preventing duplicate addresses
- ✅ Multi-chain address storage
- ✅ Authorization enforcement
- ✅ Owner management
- ✅ Address retrieval and counting
- ✅ Address removal
- ✅ Event emission

Run tests with:
```bash
cd gateway-contract
cargo test --package core
```

## Security Considerations

1. **Owner Authorization**: All critical operations require authorization from the contract owner
2. **Duplicate Prevention**: The contract prevents storing the same address twice on the same chain
3. **Error Handling**: All functions properly handle and report errors
4. **Event Emission**: All state changes emit events for off-chain tracking

## Usage Example

```rust
// Initialize the contract
let owner = Address::generate(&env);
ChainRegistry::initialize(&env, owner.clone());

// Add Ethereum address
owner.require_auth();
ChainRegistry::add_chain_address(
    &env,
    ChainId::Ethereum,
    String::from_str(&env, "0x742d35Cc6634C0532925a3b844Bc9e7595f42e11"),
    String::from_str(&env, "My Main Wallet"),
)?;

// Add Bitcoin address
owner.require_auth();
ChainRegistry::add_chain_address(
    &env,
    ChainId::Bitcoin,
    String::from_str(&env, "1A1z7agoat2GNXRN2cw46f8KU7nhs12D65"),
    String::from_str(&env, "Bitcoin Cold Storage"),
)?;

// Query addresses
let eth_count = ChainRegistry::get_chain_address_count(&env, ChainId::Ethereum);
let btc_addresses = ChainRegistry::get_chain_addresses(&env, ChainId::Bitcoin);

// Check if address exists
let has_eth = ChainRegistry::has_chain_address(
    &env,
    ChainId::Ethereum,
    String::from_str(&env, "0x742d35Cc6634C0532925a3b844Bc9e7595f42e11"),
);
```

## Integration with Alien Gateway

This `core` contract can be integrated with the main Alien Gateway contract to:
1. Maintain a registry of user addresses across multiple chains
2. Enable cross-chain payment routing
3. Support multi-chain user identity

## License

Part of the Alien Gateway project.
