# Username Encoding Format Documentation

## Overview

This document describes the username encoding format used in the Alien Gateway ZK circuits, specifically for the `username_leaf.circom` component.

## Encoding Specification

### Format
- **Type**: 32-byte array
- **Character encoding**: ASCII values (0-255)
- **Padding**: Zero-padded for unused bytes
- **Maximum length**: 32 characters

### Example: "amar"

The username "amar" is encoded as:
```json
[
  97, 109, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
]
```

Where:
- `97` = ASCII 'a'
- `109` = ASCII 'm' 
- `97` = ASCII 'a'
- `114` = ASCII 'r'
- Remaining 28 bytes are zeros (padding)

### Character Mapping

| Character | ASCII Value |
|-----------|-------------|
| 'a'       | 97          |
| 'm'       | 109         |
| 'r'       | 114         |
| '0' (pad) | 0           |

## Implementation Details

### Circuit Input Format
The `username_leaf.circom` component expects:
```circom
signal input username[32];  // 32-element array of ASCII values
signal output leaf;         // Poseidon hash of the username
```

### Hash Algorithm
The username hash is computed using Poseidon with the following steps:

1. **Chunking**: Split the 32-byte array into 8 chunks of 4 bytes each
2. **First Round**: Hash each 4-byte chunk with Poseidon(4)
3. **Second Round**: Hash the 8 intermediate results in 2 groups of 4 with Poseidon(4)
4. **Final Round**: Hash the 2 final results with Poseidon(2)

### TypeScript Implementation
```typescript
function usernameToBytes(username: string): number[] {
    const bytes = new Array(32).fill(0);
    for (let i = 0; i < Math.min(username.length, 32); i++) {
        bytes[i] = username.charCodeAt(i);
    }
    return bytes;
}
```

## Usage Examples

### Test Cases
- `"amar"` → `[97, 109, 97, 114, 0, 0, ..., 0]`
- `"test"` → `[116, 101, 115, 116, 0, 0, ..., 0]`
- `"user123"` → `[117, 115, 101, 114, 49, 50, 51, 0, ..., 0]`
- `""` (empty) → `[0, 0, 0, 0, 0, 0, ..., 0]`

### Input File Format
```json
{
  "username": [
    97, 109, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
  ]
}
```

## Verification

The standalone test `username_leaf_test.ts` verifies that:
1. The circuit compiles independently
2. The circuit output matches the TypeScript Poseidon computation
3. Multiple test cases produce consistent results

Run the test with:
```bash
npm run compile:username_leaf
npm run test:username_leaf
```

## Security Considerations

- **Fixed Length**: 32-byte fixed length prevents length-based attacks
- **Zero Padding**: Ensures deterministic hashing regardless of actual username length
- **ASCII Only**: Limited to ASCII characters for consistency across implementations
- **Deterministic**: Same username always produces the same hash

## Integration

This encoding format is used by:
- `username_leaf.circom` - Leaf generation for Merkle trees
- `merkle_inclusion.circom` - Username inclusion proofs
- `username_merkle.circom` - Username-based Merkle operations

The format is consistent across all ZK circuits in the Alien Gateway protocol.
