import assert from "node:assert/strict";
import test from "node:test";

import { Keypair, Networks, TransactionBuilder } from "@stellar/stellar-sdk";

import { StellarTxBuilder } from "../tx";

const env = {
  rpcUrl: process.env.STELLAR_RPC_URL,
  networkPassphrase: process.env.STELLAR_NETWORK_PASSPHRASE ?? Networks.TESTNET,
  contractAddress: process.env.STELLAR_CORE_CONTRACT_ADDRESS,
  sourceSecret: process.env.STELLAR_SOURCE_SECRET,
};

const shouldSkip = !env.rpcUrl || !env.contractAddress || !env.sourceSecret;

test(
  "StellarTxBuilder builds valid Soroban XDR envelopes",
  {
    skip: shouldSkip
      ? "Set STELLAR_RPC_URL, STELLAR_CORE_CONTRACT_ADDRESS, and STELLAR_SOURCE_SECRET to run this integration test"
      : false,
  },
  async () => {
    const source = Keypair.fromSecret(env.sourceSecret!).publicKey();
    const builder = new StellarTxBuilder({
      rpcUrl: env.rpcUrl!,
      networkPassphrase: env.networkPassphrase,
      contractAddress: env.contractAddress!,
      defaultSource: source,
    });

    const bytes32 = new Uint8Array(32).fill(7);

    const registerTx = await builder.buildRegister({
      caller: source,
      commitment: bytes32,
    });
    const addAddressTx = await builder.buildAddStellarAddress({
      caller: source,
      usernameHash: bytes32,
      stellarAddress: source,
    });
    const resolveTx = await builder.buildResolve(bytes32);

    for (const built of [registerTx, addAddressTx, resolveTx]) {
      const parsed = TransactionBuilder.fromXDR(built.xdr, env.networkPassphrase);
      assert.equal(parsed.toXDR(), built.xdr);
      assert.ok(built.xdr.length > 0);
    }
  },
);
