import type { Transaction } from "@stellar/stellar-sdk";

export type Bytes32Input = string | Uint8Array;
export type BinaryInput = string | Uint8Array;

export interface StellarTxBuilderConfig {
  rpcUrl: string;
  networkPassphrase: string;
  contractAddress: string;
  allowHttp?: boolean;
  defaultFee?: string;
  timeoutInSeconds?: number;
  defaultSource?: string;
}

export interface TxBuildOptions {
  fee?: string;
  timeoutInSeconds?: number;
  source?: string;
}

export interface RegisterParams extends TxBuildOptions {
  caller: string;
  commitment: Bytes32Input;
}

export interface PublicSignalsInput {
  oldRoot: Bytes32Input;
  newRoot: Bytes32Input;
}

export interface RegisterResolverParams extends TxBuildOptions {
  caller: string;
  commitment: Bytes32Input;
  proof: BinaryInput;
  publicSignals: PublicSignalsInput;
}

export interface AddStellarAddressParams extends TxBuildOptions {
  caller: string;
  usernameHash: Bytes32Input;
  stellarAddress: string;
}

export interface ResolveParams extends TxBuildOptions {
  usernameHash: Bytes32Input;
}

export interface BuiltTransaction {
  transaction: Transaction;
  xdr: string;
  method: "register" | "register_resolver" | "add_stellar_address" | "resolve_stellar";
  source: string;
}

export interface SubmitTransactionOptions {
  pollUntilSuccess?: boolean;
}
