import assert from "node:assert/strict";
import test, { beforeEach, afterEach } from "node:test";
import { UsernameResolver, UsernameNotFoundError, NoAddressLinkedError, hashUsername } from "../index";

// Mock configuration
const config = {
  network: "testnet" as const,
  rpcUrl: "https://testnet.stellar.org/rpc",
  contractId: "CDABC123",
};

// Mock fetch globally for the test
const originalFetch = global.fetch;

test.describe("UsernameResolver", () => {
  let resolver: UsernameResolver;

  beforeEach(() => {
    resolver = new UsernameResolver(config);
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  test("resolve('alice') returns the correct Stellar address from RPC mock", async () => {
    const expectedAddress = "GABC-ALICE";
    const aliceHash = await hashUsername("alice");

    global.fetch = (async () => {
      return {
        ok: true,
        json: async () => ({
          result: { address: expectedAddress },
        }),
      } as any;
    }) as any;

    const address = await resolver.resolve("alice");
    assert.strictEqual(address, expectedAddress);
  });

  test("resolveWithMemo('bob') returns both address and memo", async () => {
    const expectedAddress = "GABC-BOB";
    const expectedMemo = "42";

    global.fetch = (async () => {
      return {
        ok: true,
        json: async () => ({
          result: { address: expectedAddress, memo: expectedMemo },
        }),
      } as any;
    }) as any;

    const result = await resolver.resolveWithMemo("bob");
    assert.strictEqual(result.address, expectedAddress);
    assert.strictEqual(result.memo, expectedMemo);
  });

  test("throws UsernameNotFoundError for an unregistered user (Contract Error #1)", async () => {
    global.fetch = (async () => {
      return {
        ok: true,
        json: async () => ({
          result: { error: "Error(Contract, #1)" },
        }),
      } as any;
    }) as any;

    await assert.rejects(
      async () => resolver.resolve("notfound"),
      (error: any) => {
        return error instanceof UsernameNotFoundError && error.username === "notfound";
      }
    );
  });

  test("throws NoAddressLinkedError for a user without linked address (Contract Error #2)", async () => {
    global.fetch = (async () => {
      return {
        ok: true,
        json: async () => ({
          result: { error: "Error(Contract, #2)" },
        }),
      } as any;
    }) as any;

    await assert.rejects(
      async () => resolver.resolve("noaddress"),
      (error: any) => {
        return error instanceof NoAddressLinkedError && error.username === "noaddress";
      }
    );
  });

  test("throws standard error for other RPC failures", async () => {
    const errorMessage = "Internal Server Error";
    global.fetch = (async () => {
      return {
        ok: false,
        statusText: errorMessage,
      } as any;
    }) as any;

    await assert.rejects(
      async () => resolver.resolve("any"),
      (error: any) => error.message.includes(errorMessage)
    );
  });
});
