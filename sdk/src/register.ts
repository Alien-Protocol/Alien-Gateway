import { ProofGenerationError, TransactionFailedError, UsernameUnavailableError } from "./errors";
import { bigintToBytes32, encodeUsername, hashUsername } from "./hash";
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
  resolveUsername(commitment: string, options: { network: "testnet" | "mainnet" }): Promise<ResolveUsernameResult | null>;
  getRegistrationProofInput(
    params: { username: string; commitment: string; network: "testnet" | "mainnet" },
  ): Promise<NonInclusionInput>;
  getNonInclusionProver(): Promise<NonInclusionProver> | NonInclusionProver;
  buildRegisterResolverTransaction(params: RegisterTransactionParams): Promise<unknown>;
  signTransaction(transaction: unknown, params: { network: "testnet" | "mainnet" }): Promise<unknown>;
  submitTransaction(transaction: unknown, params: { network: "testnet" | "mainnet" }): Promise<SubmittedTransaction>;
  pollTransaction(txHash: string, params: { network: "testnet" | "mainnet" }): Promise<TransactionStatus>;
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
export async function registerUsername(
  username: string,
  wallet: WalletAdapter,
  opts: RegisterOpts = {},
): Promise<RegisterResult> {
  const network = opts.network ?? "testnet";
  const commitment = await hashUsername(username);

  const existingResolution = await wallet.resolveUsername(commitment, { network });
  if (existingResolution) {
    throw new UsernameUnavailableError(username, commitment);
  }

  const proofInput = await wallet.getRegistrationProofInput({ username, commitment, network });
  const prover = await wallet.getNonInclusionProver();

  let proofResult: NonInclusionProofResult;
  try {
    proofResult = await prover.proveNonInclusion({
      ...proofInput,
      username: encodeUsername(username).map((signal) => signal.toString()),
    });
  } catch (error) {
    throw new ProofGenerationError("Failed to generate the non-inclusion proof.", { cause: error });
  }

  if (proofResult.publicSignals[2] !== "1") {
    throw new UsernameUnavailableError(username, commitment);
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
    throw new TransactionFailedError(status.error ?? "The registration transaction failed.", txHash);
  }

  return {
    commitment,
    txHash,
    explorerUrl: buildExplorerUrl(txHash, network),
  };
}

function toRegisterPublicSignals(signals: NonInclusionProofResult["publicSignals"]): RegisterPublicSignals {
  return {
    oldRoot: bigintToBytes32(BigInt(signals[0])),
    newRoot: bigintToBytes32(BigInt(signals[1])),
  };
}

function buildExplorerUrl(txHash: string, network: "testnet" | "mainnet"): string {
  const networkPath = network === "mainnet" ? "public" : "testnet";
  return `https://stellar.expert/explorer/${networkPath}/tx/${txHash}`;
}
