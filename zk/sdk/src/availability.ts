import { hashUsername } from "./usernameHasher";
import { MerkleProofGenerator } from "./proof";

export interface SMTData {
  nodes: any;
  depth: number;
}

/**
 * Checks if a username is available using a zk non-inclusion proof.
 */
export async function isUsernameAvailable(
  username: string,
  smtRoot: bigint,
  merkleTree: SMTData,
  config: any // should be MerkleProofGeneratorConfig
): Promise<boolean> {
  try {
    // 1. Hash username into field element
    const usernameHash = hashUsername(username);

    // 2. Build circuit input (THIS replaces missing function)
    const input = buildNonInclusionInput(
      usernameHash,
      smtRoot,
      merkleTree
    );

    // 3. Generate proof
    const generator = new MerkleProofGenerator(config);

    const { proof, publicSignals } =
      await generator.proveNonInclusion(input);

    // 4. Verify proof
    const vKey = await fetchVerificationKey();
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
