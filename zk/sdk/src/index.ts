export { MerkleProofGenerator } from "./proof";
export { bigintToBytes32, encodeUsername, hashUsername } from "./hash";
export { UsernameResolver } from "./resolver";
export type { NetworkConfig, ResolveWithMemoResult } from "./resolver";
export {
  AlienGatewayError,
  NoAddressLinkedError,
  ProofGenerationError,
  TransactionFailedError,
  UsernameNotFoundError,
  UsernameUnavailableError,
} from "./errors";
export { registerUsername } from "./register";
export type {
  CircuitArtifactPaths,
  Groth16Proof,
  InclusionInput,
  InclusionProofResult,
  InclusionPublicSignals,
  MerkleProofGeneratorConfig,
  NonInclusionInput,
  NonInclusionProofResult,
  NonInclusionPublicSignals,
  SignalInput,
} from "./types";
export type {
  NonInclusionProver,
  RegisterOpts,
  RegisterPublicSignals,
  RegisterResult,
  RegisterTransactionParams,
  ResolveUsernameResult,
  SubmittedTransaction,
  TransactionStatus,
  WalletAdapter,
} from "./register";
