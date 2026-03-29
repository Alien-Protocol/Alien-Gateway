# @alien-protocol/sdk

TypeScript SDK for Alien Protocol - username resolution, proof generation, and payment transactions.

## Installation

```bash
npm install @alien-protocol/sdk
```

## Usage

### Generating Inclusion Proofs

```typescript
import { MerkleProofGenerator } from '@alien-protocol/sdk';

const generator = new MerkleProofGenerator({
  inclusion: {
    wasmPath: './circuits/inclusion.wasm',
    zkeyPath: './circuits/inclusion.zkey'
  },
  nonInclusion: {
    wasmPath: './circuits/non_inclusion.wasm',
    zkeyPath: './circuits/non_inclusion.zkey'
  }
});

// Prove username inclusion
const inclusionProof = await generator.proveInclusion({
  username: [/* username hash as array */],
  pathElements: [/* merkle path elements */],
  pathIndices: [/* merkle path indices */],
  root: '0x...' // merkle root
});

console.log('Proof:', inclusionProof.proof);
console.log('Public Signals:', inclusionProof.publicSignals);
```

### Generating Non-Inclusion Proofs

```typescript
// Prove username is available (non-inclusion)
const nonInclusionProof = await generator.proveNonInclusion({
  username: [/* username hash as array */],
  leaf_before: '0x...',
  leaf_after: '0x...',
  merklePathBeforeSiblings: [/* siblings */],
  merklePathBeforeIndices: [/* indices */],
  merklePathAfterSiblings: [/* siblings */],
  merklePathAfterIndices: [/* indices */],
  root: '0x...'
});

console.log('Availability Proof:', nonInclusionProof.proof);
console.log('Is Available:', nonInclusionProof.publicSignals[2]);
```

## Types

The SDK exports the following TypeScript types:

- `MerkleProofGenerator` - Main class for proof generation
- `InclusionInput` - Input for inclusion proofs
- `NonInclusionInput` - Input for non-inclusion proofs
- `InclusionProofResult` - Result of inclusion proof generation
- `NonInclusionProofResult` - Result of non-inclusion proof generation
- `Groth16Proof` - Groth16 proof structure
- `CircuitArtifactPaths` - Paths to circuit artifacts
- `MerkleProofGeneratorConfig` - Configuration for proof generator

## Building

```bash
npm run build
```

## License

MIT
