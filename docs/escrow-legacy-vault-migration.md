# Escrow LegacyVault Migration Note

`LegacyVault` exists only to support older escrow storage that used
`DataKey::Vault(BytesN<32>)` before vault data was split into:

- `DataKey::VaultConfig(BytesN<32>)`
- `DataKey::VaultState(BytesN<32>)`

The fallback remains necessary because these snapshot fixtures still serialize
the legacy key:

- `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_setup_success.1.json`
- `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_trigger_success.1.json`
- `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_second_cycle_success.1.json`
- `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_insufficient_balance_panics.1.json`
- `gateway-contract/contracts/escrow_contract/test_snapshots/test/test_auto_pay_early_trigger_panics.1.json`

It is safe to remove `LegacyVault` only after a migration rewrites every stored
`DataKey::Vault(BytesN<32>)` record into the split `VaultConfig` and
`VaultState` entries, and the snapshot fixtures above are regenerated to match.
