import type { MerkleProofGeneratorConfig } from "./types";
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
export declare function isUsernameAvailable(username: string, smtRoot: bigint, merkleTree: SMTData, config: UsernameAvailabilityConfig): Promise<boolean>;
