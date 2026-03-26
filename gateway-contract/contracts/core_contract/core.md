# Core Contract Notes

## SMT Root Sequencing Requirement

`register_resolver` enforces strict root sequencing:

- `public_signals.old_root` must exactly equal the current on-chain SMT root.
- A successful `register_resolver` updates the on-chain root to `public_signals.new_root`.
- Any later call reusing the pre-update root is rejected as stale.

This replay protection prevents re-submitting proofs against an already-consumed root. In tests, the stale replay path is asserted to panic with `Error(Contract, #4)` (`StaleRoot`).

