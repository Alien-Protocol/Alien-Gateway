"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isUsernameAvailable = isUsernameAvailable;
const snarkjs_1 = __importDefault(require("snarkjs"));
const proof_1 = require("./proof");
const hash_1 = require("./hash");
/**
 * Checks if a username is available using a zk non-inclusion proof.
 */
async function isUsernameAvailable(username, smtRoot, merkleTree, config) {
    // 1. Hash username
    const usernameHash = (0, hash_1.hashUsername)(username);
    // 2. Build circuit input (still your responsibility)
    // @ts-ignore - buildNonInclusionInput is a helper function to be provided by user
    const input = buildNonInclusionInput(usernameHash, smtRoot, merkleTree);
    // 3. Use SDK proof generator (✅ no hardcoded paths)
    const generator = new proof_1.MerkleProofGenerator(config.proofConfig);
    const { proof, publicSignals } = await generator.proveNonInclusion(input);
    // 4. Verify using configurable vkey path
    // @ts-ignore - fetchVerificationKey is a helper function to be provided by user
    const vKey = await fetchVerificationKey(config.vkeyPath);
    const isValid = await snarkjs_1.default.groth16.verify(vKey, publicSignals, proof);
    return isValid;
}
