import assert from "node:assert/strict";
import test from "node:test";

import {
  ProofGenerationError,
  TransactionFailedError,
  UsernameUnavailableError,
  hashUsername,
  registerUsername,
} from "../index";
import type {
  NonInclusionProver,
  NonInclusionInput,
  NonInclusionProofResult,
  RegisterTransactionParams,
  ResolveUsernameResult,
  SubmittedTransaction,
  TransactionStatus,
  WalletAdapter,
} from "../index";

class MockWalletAdapter implements WalletAdapter {
  public constructor(
    private readonly overrides: Partial<WalletAdapter> = {},
    private readonly proofResult: NonInclusionProofResult = {
      proof: {
        pi_a: ["1", "2", "1"],
        pi_b: [["1", "2"], ["3", "4"], ["1", "0"]],
        pi_c: ["1", "2", "1"],
        protocol: "groth16",
        curve: "bn128",
      },
      publicSignals: ["11", "22", "1"],
    },
  ) {}

  public async resolveUsername(
    commitment: string,
    options: { network: "testnet" | "mainnet" },
  ): Promise<ResolveUsernameResult | null> {
    return this.overrides.resolveUsername
      ? this.overrides.resolveUsername(commitment, options)
      : null;
  }

  public async getRegistrationProofInput(
    params: { username: string; commitment: string; network: "testnet" | "mainnet" },
  ): Promise<NonInclusionInput> {
    return this.overrides.getRegistrationProofInput
      ? this.overrides.getRegistrationProofInput(params)
      : {
          username: new Array<string>(32).fill("0"),
          leaf_before: "1",
          leaf_after: "3",
          merklePathBeforeSiblings: new Array<string>(20).fill("0"),
          merklePathBeforeIndices: new Array<string>(20).fill("0"),
          merklePathAfterSiblings: new Array<string>(20).fill("0"),
          merklePathAfterIndices: new Array<string>(20).fill("0"),
          root: "11",
        };
  }

  public getNonInclusionProver(): NonInclusionProver | Promise<NonInclusionProver> {
    if (this.overrides.getNonInclusionProver) {
      return this.overrides.getNonInclusionProver();
    }

    return {
      proveNonInclusion: async () => this.proofResult,
    };
  }

  public async buildRegisterResolverTransaction(params: RegisterTransactionParams): Promise<unknown> {
    return this.overrides.buildRegisterResolverTransaction
      ? this.overrides.buildRegisterResolverTransaction(params)
      : { kind: "unsigned", params };
  }

  public async signTransaction(
    transaction: unknown,
    params: { network: "testnet" | "mainnet" },
  ): Promise<unknown> {
    return this.overrides.signTransaction
      ? this.overrides.signTransaction(transaction, params)
      : { kind: "signed", transaction };
  }

  public async submitTransaction(
    transaction: unknown,
    params: { network: "testnet" | "mainnet" },
  ): Promise<SubmittedTransaction> {
    return this.overrides.submitTransaction
      ? this.overrides.submitTransaction(transaction, params)
      : { txHash: "abc123" };
  }

  public async pollTransaction(
    txHash: string,
    params: { network: "testnet" | "mainnet" },
  ): Promise<TransactionStatus> {
    return this.overrides.pollTransaction
      ? this.overrides.pollTransaction(txHash, params)
      : { status: "success" };
  }
}

test("registerUsername completes the full testnet flow", async () => {
  const adapter = new MockWalletAdapter();
  const result = await registerUsername("amar", adapter, { network: "testnet" });

  assert.equal(result.txHash, "abc123");
  assert.equal(result.explorerUrl, "https://stellar.expert/explorer/testnet/tx/abc123");
  assert.equal(result.commitment, await hashUsername("amar"));
});

test("registerUsername throws UsernameUnavailableError when the commitment already resolves", async () => {
  const adapter = new MockWalletAdapter({
    resolveUsername: async () => ({ wallet: "GABC" }),
  });

  await assert.rejects(
    () => registerUsername("amar", adapter),
    (error: unknown) =>
      error instanceof UsernameUnavailableError && error.username === "amar",
  );
});

test("registerUsername throws UsernameUnavailableError when the proof marks the username unavailable", async () => {
  const adapter = new MockWalletAdapter(
    {},
    {
      proof: {
        pi_a: ["1", "2", "1"],
        pi_b: [["1", "2"], ["3", "4"], ["1", "0"]],
        pi_c: ["1", "2", "1"],
        protocol: "groth16",
        curve: "bn128",
      },
      publicSignals: ["11", "22", "0"],
    },
  );

  await assert.rejects(() => registerUsername("amar", adapter), UsernameUnavailableError);
});

test("registerUsername throws ProofGenerationError if the prover fails", async () => {
  const adapter = new MockWalletAdapter({
    getNonInclusionProver: () => ({
      proveNonInclusion: async () => {
        throw new Error("snarkjs exploded");
      },
    }),
  });

  await assert.rejects(() => registerUsername("amar", adapter), ProofGenerationError);
});

test("registerUsername throws TransactionFailedError if confirmation fails", async () => {
  const adapter = new MockWalletAdapter({
    pollTransaction: async () => ({ status: "failed", error: "Contract reverted" }),
  });

  await assert.rejects(() => registerUsername("amar", adapter), TransactionFailedError);
});

test(
  "registerUsername live testnet flow",
  { skip: "Requires a real Stellar testnet wallet adapter and proof service." },
  async () => {
    assert.fail("Provide a concrete Soroban wallet adapter and remove the skip to run live.");
  },
);
