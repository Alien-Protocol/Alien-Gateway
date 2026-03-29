"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerUsername = registerUsername;
const errors_1 = require("./errors");
const hash_1 = require("./hash");
/**
 * Registers an Alien Gateway username by hashing the input, proving non-inclusion,
 * submitting `register_resolver`, and waiting for on-chain confirmation.
 *
 * @example
 * ```ts
 * import { registerUsername } from "./register";
 *
 * const result = await registerUsername("amar", walletAdapter, {
 *   network: "testnet",
 *   memo: 42,
 * });
 *
 * console.log(result.commitment, result.txHash, result.explorerUrl);
 * ```
 */
async function registerUsername(username, wallet, opts = {}) {
    const network = opts.network ?? "testnet";
    const commitment = await (0, hash_1.hashUsername)(username);
    const existingResolution = await wallet.resolveUsername(commitment, { network });
    if (existingResolution) {
        throw new errors_1.UsernameUnavailableError(username, commitment);
    }
    const proofInput = await wallet.getRegistrationProofInput({ username, commitment, network });
    const prover = await wallet.getNonInclusionProver();
    let proofResult;
    try {
        proofResult = await prover.proveNonInclusion({
            ...proofInput,
            username: (0, hash_1.encodeUsername)(username).map((signal) => signal.toString()),
        });
    }
    catch (error) {
        throw new errors_1.ProofGenerationError("Failed to generate the non-inclusion proof.", { cause: error });
    }
    if (proofResult.publicSignals[2] !== "1") {
        throw new errors_1.UsernameUnavailableError(username, commitment);
    }
    const publicSignals = toRegisterPublicSignals(proofResult.publicSignals);
    const unsignedTransaction = await wallet.buildRegisterResolverTransaction({
        username,
        commitment,
        proof: proofResult.proof,
        publicSignals,
        memo: opts.memo,
        stellarAddress: opts.stellarAddress,
        network,
    });
    const signedTransaction = await wallet.signTransaction(unsignedTransaction, { network });
    const { txHash } = await wallet.submitTransaction(signedTransaction, { network });
    const status = await wallet.pollTransaction(txHash, { network });
    if (status.status !== "success") {
        throw new errors_1.TransactionFailedError(status.error ?? "The registration transaction failed.", txHash);
    }
    return {
        commitment,
        txHash,
        explorerUrl: buildExplorerUrl(txHash, network),
    };
}
function toRegisterPublicSignals(signals) {
    return {
        oldRoot: (0, hash_1.bigintToBytes32)(BigInt(signals[0])),
        newRoot: (0, hash_1.bigintToBytes32)(BigInt(signals[1])),
    };
}
function buildExplorerUrl(txHash, network) {
    const networkPath = network === "mainnet" ? "public" : "testnet";
    return `https://stellar.expert/explorer/${networkPath}/tx/${txHash}`;
}
