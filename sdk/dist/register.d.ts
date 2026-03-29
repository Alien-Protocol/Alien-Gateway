import type { Groth16Proof, NonInclusionInput, NonInclusionProofResult } from "./types";
export interface RegisterOpts {
    memo?: number;
    stellarAddress?: string;
    network?: "testnet" | "mainnet";
}
export interface RegisterResult {
    commitment: string;
    txHash: string;
    explorerUrl: string;
}
export interface ResolveUsernameResult {
    wallet: string;
    memo?: number | null;
}
export interface RegisterPublicSignals {
    oldRoot: string;
    newRoot: string;
}
export interface RegisterTransactionParams {
    username: string;
    commitment: string;
    proof: Groth16Proof;
    publicSignals: RegisterPublicSignals;
    memo?: number;
    stellarAddress?: string;
    network: "testnet" | "mainnet";
}
export interface SubmittedTransaction {
    txHash: string;
}
export interface TransactionStatus {
    status: "success" | "failed";
    error?: string;
}
export interface NonInclusionProver {
    proveNonInclusion(input: NonInclusionInput): Promise<NonInclusionProofResult>;
}
export interface WalletAdapter {
    resolveUsername(commitment: string, options: {
        network: "testnet" | "mainnet";
    }): Promise<ResolveUsernameResult | null>;
    getRegistrationProofInput(params: {
        username: string;
        commitment: string;
        network: "testnet" | "mainnet";
    }): Promise<NonInclusionInput>;
    getNonInclusionProver(): Promise<NonInclusionProver> | NonInclusionProver;
    buildRegisterResolverTransaction(params: RegisterTransactionParams): Promise<unknown>;
    signTransaction(transaction: unknown, params: {
        network: "testnet" | "mainnet";
    }): Promise<unknown>;
    submitTransaction(transaction: unknown, params: {
        network: "testnet" | "mainnet";
    }): Promise<SubmittedTransaction>;
    pollTransaction(txHash: string, params: {
        network: "testnet" | "mainnet";
    }): Promise<TransactionStatus>;
}
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
export declare function registerUsername(username: string, wallet: WalletAdapter, opts?: RegisterOpts): Promise<RegisterResult>;
