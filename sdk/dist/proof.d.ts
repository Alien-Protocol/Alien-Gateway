import type { InclusionInput, InclusionProofResult, MerkleProofGeneratorConfig, NonInclusionInput, NonInclusionProofResult } from "./types";
export declare class MerkleProofGenerator {
    private readonly config;
    constructor(config: MerkleProofGeneratorConfig);
    proveInclusion(input: InclusionInput): Promise<InclusionProofResult>;
    proveNonInclusion(input: NonInclusionInput): Promise<NonInclusionProofResult>;
}
