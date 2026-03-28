# Alien Gateway Contracts

This workspace contains the smart contracts for the Alien Gateway protocol on the Stellar network. The contracts are built using Soroban SDK and provide core functionality for decentralized alien asset trading and auctions.

## Workspace Structure

The gateway-contract workspace consists of the following contracts:

- **`core_contract`** - Main gateway contract handling core protocol logic
- **`escrow_contract`** - Escrow service for secure asset holding during transactions
- **`factory_contract`** - Factory contract for deploying new instances of gateway contracts
- **`auction_contract`** - Auction mechanism for alien asset trading
- **`tests/`** - Integration tests for the entire contract suite

Each contract is a separate crate within the workspace and can be built and tested independently or as part of the entire workspace.

## Prerequisites

- Rust 1.70+ with `wasm32v1-none` target
- Stellar CLI for deployment and interaction
- Soroban CLI for local testing

### Install Rust Target

```bash
rustup target add wasm32v1-none
```

### Install Stellar CLI

```bash
# Install Stellar CLI
cargo install --locked stellar-cli

# Or download from GitHub releases
# https://github.com/stellar/stellar-cli/releases
```

## Build Instructions

### Build All Contracts

To build all contracts in the workspace for release:

```bash
# Build all contracts optimized for WASM
cargo build --target wasm32v1-none --release

# Alternative using Stellar CLI
stellar contract build
```

### Build Individual Contracts

```bash
# Build specific contract
cargo build --target wasm32v1-none --release -p core_contract
cargo build --target wasm32v1-none --release -p escrow_contract
cargo build --target wasm32v1-none --release -p factory_contract
cargo build --target wasm32v1-none --release -p auction_contract
```

The compiled WASM files will be located in:
```
target/wasm32v1-none/release/
├── core_contract.wasm
├── escrow_contract.wasm
├── factory_contract.wasm
└── auction_contract.wasm
```

## Test Instructions

### Run All Tests

```bash
# Run all unit and integration tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Quality Checks

```bash
# Run Clippy for linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Test Individual Contracts

```bash
# Test specific contract
cargo test -p core_contract
cargo test -p escrow_contract
cargo test -p factory_contract
cargo test -p auction_contract

# Run integration tests
cargo test -p tests
```

## Deployment Instructions

### Prerequisites for Deployment

1. **Stellar Testnet Account**: Create a testnet account at [Stellar Laboratory](https://laboratory.stellar.org/)
2. **Get Testnet Lumens**: Fund your account using the [Stellar Testnet Faucet](https://friendbot.stellar.org/)
3. **Configure Stellar CLI**: Set up your network and account

### Configure Stellar CLI

```bash
# Set network to testnet
stellar --network testnet

# Add your secret key (use environment variables for security)
export STELLAR_SECRET_KEY="your_secret_key_here"

# Or use a config file
stellar config set network testnet
stellar config set secret-key your_secret_key_here
```

### Deploy Contracts to Testnet

#### Deploy Core Contract

```bash
# Deploy core contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/core_contract.wasm \
  --network testnet \
  --source-account your_public_key_here

# Note the contract ID from the output
```

#### Deploy Supporting Contracts

```bash
# Deploy escrow contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/escrow_contract.wasm \
  --network testnet \
  --source-account your_public_key_here

# Deploy factory contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/factory_contract.wasm \
  --network testnet \
  --source-account your_public_key_here

# Deploy auction contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/auction_contract.wasm \
  --network testnet \
  --source-account your_public_key_here
```

### Verify Deployment

```bash
# Check contract status
stellar contract info \
  --contract-id your_contract_id_here \
  --network testnet

# Read contract ledger entries
stellar contract read \
  --contract-id your_contract_id_here \
  --network testnet
```

## Development Workflow

### 1. Make Changes

Edit contract source code in the respective `contracts/*/src/` directories.

### 2. Run Tests

```bash
cargo test
cargo clippy
cargo fmt
```

### 3. Build

```bash
cargo build --target wasm32v1-none --release
```

### 4. Deploy to Testnet

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/your_contract.wasm --network testnet
```

## Useful Commands

### Contract Interaction

```bash
# Invoke contract method
stellar contract invoke \
  --contract-id your_contract_id_here \
  --method your_method_name \
  --arg1 value1 \
  --arg2 value2 \
  --network testnet

# Get contract ledger entries
stellar contract read \
  --contract-id your_contract_id_here \
  --network testnet
```

### Network Management

```bash
# Switch between networks
stellar config set network testnet
stellar config set network public
stellar config set network future

# View current configuration
stellar config show
```

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure you have the correct Rust target installed:
   ```bash
   rustup target add wasm32v1-none
   ```

2. **Testnet Funding**: Use the friendbot to fund your testnet account:
   ```bash
   curl "https://friendbot.stellar.org?addr=your_public_key_here"
   ```

3. **Contract Size**: Stellar has a 32KB contract size limit. Use release builds with optimization.

4. **Gas Fees**: Ensure your testnet account has enough lumens for deployment and transactions.

### Debug Mode

For development with logging, use the release-with-logs profile:

```bash
cargo build --target wasm32v1-none --profile release-with-logs
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run all tests and quality checks
5. Submit a pull request

Ensure all tests pass and code follows the project's style guidelines before submitting.

## License

This project is licensed under the MIT License - see the main repository for details.