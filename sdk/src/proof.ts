// @ts-ignore - snarkjs doesn't have type definitions
import snarkjs from "snarkjs";

import type {
  Groth16Proof,
  InclusionInput,
  InclusionProofResult,
  MerkleProofGeneratorConfig,
  NonInclusionInput,
  NonInclusionProofResult,
  SignalInput,
} from "./types";

type NormalizedSignal = string | NormalizedSignal[] | { [key: string]: NormalizedSignal };

export class MerkleProofGenerator {
  public constructor(private readonly config: MerkleProofGeneratorConfig) {}

  public async proveInclusion(input: InclusionInput): Promise<InclusionProofResult> {
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      normalizeInput(input),
      this.config.inclusion.wasmPath,
      this.config.inclusion.zkeyPath,
    );

    return {
      proof: proof as Groth16Proof,
      publicSignals: publicSignals as InclusionProofResult["publicSignals"],
    };
  }

  public async proveNonInclusion(input: NonInclusionInput): Promise<NonInclusionProofResult> {
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      normalizeInput(input),
      this.config.nonInclusion.wasmPath,
      this.config.nonInclusion.zkeyPath,
    );

    return {
      proof: proof as Groth16Proof,
      publicSignals: publicSignals as NonInclusionProofResult["publicSignals"],
    };
  }
}

function normalizeInput<T extends Record<string, SignalInput | SignalInput[]>>(input: T): Record<string, NormalizedSignal> {
  return Object.fromEntries(
    Object.entries(input).map(([key, value]) => [key, normalizeSignal(value)]),
  );
}

function normalizeSignal(value: SignalInput | SignalInput[]): NormalizedSignal {
  if (Array.isArray(value)) {
    return value.map((entry) => normalizeSignal(entry));
  }

  return typeof value === "bigint" ? value.toString() : String(value);
}
