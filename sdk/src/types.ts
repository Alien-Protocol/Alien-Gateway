export type SignalInput = string | number | bigint;

export interface CircuitArtifactPaths {
  wasmPath: string;
  zkeyPath: string;
}

export interface MerkleProofGeneratorConfig {
  inclusion: CircuitArtifactPaths;
  nonInclusion: CircuitArtifactPaths;
}

export interface InclusionInput {
  username: SignalInput[];
  pathElements: SignalInput[];
  pathIndices: SignalInput[];
  root: SignalInput;
  [key: string]: SignalInput | SignalInput[];
}

export interface NonInclusionInput {
  username: SignalInput[];
  leaf_before: SignalInput;
  leaf_after: SignalInput;
  merklePathBeforeSiblings: SignalInput[];
  merklePathBeforeIndices: SignalInput[];
  merklePathAfterSiblings: SignalInput[];
  merklePathAfterIndices: SignalInput[];
  root: SignalInput;
  [key: string]: SignalInput | SignalInput[];
}

export interface Groth16Proof {
  pi_a: string[];
  pi_b: string[][];
  pi_c: string[];
  protocol: string;
  curve: string;
}

export type InclusionPublicSignals = [root: string, outRoot: string];
export type NonInclusionPublicSignals = [root: string, outRoot: string, isAvailable: string];

export interface InclusionProofResult {
  proof: Groth16Proof;
  publicSignals: InclusionPublicSignals;
}

export interface NonInclusionProofResult {
  proof: Groth16Proof;
  publicSignals: NonInclusionPublicSignals;
}
