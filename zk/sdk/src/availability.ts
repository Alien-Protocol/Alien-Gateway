import { groth16 } from "snarkjs";
import { hashUsername } from "./usernameHasher";
import { generateNonInclusionProof } from "./merkleProofGenerator";

export interface SMTData {
  // shape depends on your tree implementation
  // keep generic for flexibility
  nodes: any;
  depth: number;
}

/**
 * Checks if a username is available using a zk non-inclusion proof.
 */
export async function isUsernameAvailable(
  username: string,
  smtRoot: bigint,
  merkleTree: SMTData
): Promise<boolean> {
  try {
    // 1. Hash username into field element
    const usernameHash = hashUsername(username);

    // 2. Generate non-inclusion witness inputs
    const input = await generateNonInclusionProof(
      usernameHash,
      smtRoot,
      merkleTree
    );

    // 3. Generate proof
    const { proof, publicSignals } = await groth16.fullProve(
      input,
      "circuits/merkle_non_inclusion.wasm",
      "circuits/merkle_non_inclusion.zkey"
    );

    // 4. Verify proof
    const vKey = await fetchVerificationKey();
    const isValid = await groth16.verify(vKey, publicSignals, proof);

    return isValid;
  } catch (err) {
    console.error("Username availability check failed:", err);
    return false;
  }
}

/**
 * Loads verification key (can be cached in production)
 */
async function fetchVerificationKey() {
  // adjust path depending on your setup
  const res = await fetch("/circuits/merkle_non_inclusion_vkey.json");
  return res.json();
}