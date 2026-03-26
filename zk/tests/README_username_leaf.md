# Username Leaf Circuit Test

## Overview

This directory contains the standalone test for the `username_leaf.circom` component, which bridges raw username integer arrays to Poseidon-hashed leaves used in Merkle proofs.

## Files

- `username_leaf_main.circom` - Standalone version of username_leaf.circom with main component
- `username_leaf_test.ts` - TypeScript test verifying circuit correctness
- `input.json` - Test input for username "amar" in 32-byte zero-padded format
- `username_encoding.md` - Detailed documentation of username encoding format

## Usage

### Prerequisites

Install dependencies:
```bash
cd zk
npm install
```

### Compilation

Compile the username_leaf circuit:
```bash
npm run compile:username_leaf
```

Or compile all circuits:
```bash
npm run compile:all
```

### Testing

Run the username leaf test:
```bash
npm run test:username_leaf
```

### Manual Testing

1. Generate witness:
```bash
node -e "
const snarkjs = require('snarkjs');
const fs = require('fs');
const input = JSON.parse(fs.readFileSync('input.json', 'utf8'));
snarkjs.wtns.calculateWitness('build/username_leaf_main/wasm/username_leaf_main_js/username_leaf_main.wasm', input)
  .then(({witness}) => console.log('Leaf hash:', witness[1].toString()));
"
```

2. Verify with TypeScript:
```bash
node -e "
const { buildPoseidon } = require('circomlibjs');
buildPoseidon().then(poseidon => {
  const username = [97, 109, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const F = poseidon.F;
  
  // Step 1: Hash in chunks of 4
  const h = [];
  for (let i = 0; i < 8; i++) {
    const chunk = [BigInt(username[i*4]), BigInt(username[i*4+1]), BigInt(username[i*4+2]), BigInt(username[i*4+3])];
    h.push(F.toObject(poseidon(chunk)));
  }
  
  // Step 2: Hash intermediate hashes
  const h2 = [];
  for (let i = 0; i < 2; i++) {
    const chunk = [h[i*4], h[i*4+1], h[i*4+2], h[i*4+3]];
    h2.push(F.toObject(poseidon(chunk)));
  }
  
  // Final hash
  const finalHash = F.toObject(poseidon([h2[0], h2[1]]));
  console.log('TypeScript hash:', finalHash.toString());
});
"
```

## Expected Results

For username "amar" (ASCII: [97, 109, 97, 114, 0, 0, ..., 0]):
- Circuit output: `20874380368079837438632997674874984863391487284332644052898098881644791571788`
- TypeScript output: Same value

## Test Cases

The test verifies multiple usernames:
- "amar" - Basic test case
- "test" - Different characters
- "user123" - Alphanumeric
- "alice" - Common name
- "" (empty) - Edge case

## Integration

This standalone test ensures that:
1. `username_leaf.circom` compiles independently
2. Circuit output matches TypeScript Poseidon computation
3. Username encoding is consistent across implementations
4. Merkle proof generation uses correct leaf values

## Troubleshooting

### Compilation Issues
- Ensure circom is installed and in PATH
- Check that node_modules contains circomlib
- Verify all include paths are correct

### Test Failures
- Check that input.json format is correct
- Verify WASM file exists and is accessible
- Ensure ts-node is properly configured

### Hash Mismatches
- Verify username encoding (32-byte, zero-padded)
- Check Poseidon implementation consistency
- Ensure chunking logic matches circuit

## CI/CD

The test is integrated into the ZK CI workflow:
- Compiles username_leaf_main circuit
- Verifies build artifacts exist
- Runs test suite (can be added as future enhancement)

## Security Notes

- Fixed 32-byte length prevents length-based attacks
- Zero padding ensures deterministic hashing
- ASCII-only encoding for cross-platform consistency
- Poseidon hash provides cryptographic security
