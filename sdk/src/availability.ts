import snarkjs from "snarkjs";
import {
  MerkleProofGenerator,
} from "./proof";
import type { MerkleProofGeneratorConfig } from "./types";
import { hashUsername } from "./hash";

export interface SMTData {
  nodes: any;
  depth: number;
}

export interface UsernameAvailabilityConfig {
  proofConfig: MerkleProofGeneratorConfig;
  vkeyPath: string;
}

/**
 * Checks if a username is available using a zk non-inclusion proof.
 */
export async function isUsernameAvailable(
  username: string,
  smtRoot: bigint,
  merkleTree: SMTData,
  config: UsernameAvailabilityConfig
): Promise<boolean> {
  // 1. Hash username
  const usernameHash = hashUsername(username);

  // 2. Build circuit input (still your responsibility)
  // @ts-ignore - buildNonInclusionInput is a helper function to be provided by user
  const input = buildNonInclusionInput(
    usernameHash,
    smtRoot,
    merkleTree
  );

  // 3. Use SDK proof generator (✅ no hardcoded paths)
  const generator = new MerkleProofGenerator(
    config.proofConfig
  );

  const { proof, publicSignals } =
    await generator.proveNonInclusion(input);

  // 4. Verify using configurable vkey path
  // @ts-ignore - fetchVerificationKey is a helper function to be provided by user
  const vKey = await fetchVerificationKey(config.vkeyPath);

  const isValid = await snarkjs.groth16.verify(
    vKey,
    publicSignals,
    proof
  );

  return isValid;
}
