# Security Fix Summary for Finding F-02

## Issue Description
The `MerkleUpdateProof` circuit in `zk/circuits/merkle/merkle_update_proof.circom` accepted `usernameHash` as an unconstrained private input, allowing provers to supply arbitrary field elements as leaves in the Merkle tree.

## Security Vulnerability
- **Finding F-02**: `usernameHash` private input was unconstrained
- **Impact**: Provers could insert non-canonical data into the registry
- **Root Cause**: Circuit did not verify that `usernameHash` was produced by `UsernameHash()`

## Fix Implementation

### 1. Updated Circuit Interface
**Before:**
```circom
signal input usernameHash;  // Unconstrained field element
```

**After:**
```circom
signal input username[32];  // Constrained username array
```

### 2. Internal Hash Computation
- Added `include "../username_hash.circom"` 
- Instantiated `UsernameHash()` component internally
- Connected username array to hash computation
- Used computed hash as leaf value

### 3. Updated Test Suite
- Modified `test_update_proof.js` to provide `username[32]` array
- Added `computeUsernameHash()` helper function matching circuit algorithm
- Updated test inputs to use new interface

## Files Changed
1. `zk/circuits/merkle/merkle_update_proof.circom`
2. `zk/tests/test_update_proof.js`

## Security Benefits
- **Constrained Input**: Username must be provided as 32-character array
- **Canonical Hashing**: Hash is computed internally using `UsernameHash()`
- **Prevents Arbitrary Data**: Provers cannot inject arbitrary field elements
- **Maintains Compatibility**: Same public interface and security guarantees

## Verification
The fix ensures that:
1. Only properly hashed usernames can be inserted into the Merkle tree
2. The hash computation is constrained within the circuit
3. All existing security properties are preserved
4. Test suite validates the new input format

This addresses Finding F-02 from the threat model completely.
