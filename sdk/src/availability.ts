import snarkjs from "snarkjs";
import {
  MerkleProofGenerator,
  MerkleProofGeneratorConfig,
} from "./proof";
import { hashUsername } from "./usernameHasher";

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
  try {
    // 1. Hash username
    const usernameHash = hashUsername(username);

    // 2. Build circuit input (still your responsibility)
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
    const vKey = await fetchVerificationKey(config.vkeyPath);

    const isValid = await snarkjs.groth16.verify(
      vKey,
      publicSignals,
      proof
    );

    return isValid;
  } catch (err) {
    console.error("Username availability check failed:", err);
    return false;
  }
}
