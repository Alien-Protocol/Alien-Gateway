import assert from "node:assert/strict";
import test from "node:test";

import { FreighterAdapter, WalletDetectionError, XBullAdapter, autoDetectWallet } from "../index";

type BrowserWindow = typeof globalThis & {
  window?: {
    xBullSDK?: {
      connect?: () => Promise<void>;
      requestAccess?: () => Promise<void>;
      getPublicKey?: () => Promise<string>;
      getAddress?: () => Promise<string>;
      signTransaction?: (xdr: string) => Promise<string | { signedXdr?: string }>;
      sign?: (xdr: string) => Promise<string | { signedXdr?: string }>;
    };
    xBull?: {
      connect?: () => Promise<void>;
      requestAccess?: () => Promise<void>;
      getPublicKey?: () => Promise<string>;
      getAddress?: () => Promise<string>;
      signTransaction?: (xdr: string) => Promise<string | { signedXdr?: string }>;
      sign?: (xdr: string) => Promise<string | { signedXdr?: string }>;
    };
  };
};

const browserGlobal = globalThis as BrowserWindow;

test("FreighterAdapter connect/getPublicKey/signTransaction use the wrapped API", async () => {
  let accessRequested = false;

  const adapter = new FreighterAdapter({
    requestAccess: async () => {
      accessRequested = true;
      return "GCFREIGHTERPUBLICKEY";
    },
    getPublicKey: async () => "GCFREIGHTERPUBLICKEY",
    signTransaction: async (xdr: string) => ({ signedTxXdr: `${xdr}-signed` }),
  });

  await adapter.connect();

  assert.equal(accessRequested, true);
  assert.equal(await adapter.getPublicKey(), "GCFREIGHTERPUBLICKEY");
  assert.equal(await adapter.signTransaction("AAAA"), "AAAA-signed");
});

test("XBullAdapter connect/getPublicKey/signTransaction use the wrapped provider", async () => {
  let connectCalled = false;

  const adapter = new XBullAdapter({
    connect: async () => {
      connectCalled = true;
    },
    getPublicKey: async () => "GCXBULLPUBLICKEY",
    signTransaction: async (xdr: string) => ({ signedXdr: `${xdr}-xbull` }),
  });

  await adapter.connect();

  assert.equal(connectCalled, true);
  assert.equal(await adapter.getPublicKey(), "GCXBULLPUBLICKEY");
  assert.equal(await adapter.signTransaction("BBBB"), "BBBB-xbull");
});

test("autoDetectWallet prefers Freighter when it is available", async () => {
  const originalWindow = browserGlobal.window;
  const freighterAvailability = FreighterAdapter.isAvailable;

  browserGlobal.window = {
    xBullSDK: {
      getPublicKey: async () => "GCXBULLPUBLICKEY",
      signTransaction: async (xdr: string) => xdr,
    },
  };

  FreighterAdapter.isAvailable = async () => true;

  try {
    const adapter = await autoDetectWallet();
    assert.equal(adapter instanceof FreighterAdapter, true);
  } finally {
    FreighterAdapter.isAvailable = freighterAvailability;
    browserGlobal.window = originalWindow;
  }
});

test("autoDetectWallet falls back to xBull when Freighter is unavailable", async () => {
  const originalWindow = browserGlobal.window;
  const freighterAvailability = FreighterAdapter.isAvailable;

  browserGlobal.window = {
    xBullSDK: {
      getPublicKey: async () => "GCXBULLPUBLICKEY",
      signTransaction: async (xdr: string) => xdr,
    },
  };

  FreighterAdapter.isAvailable = async () => false;

  try {
    const adapter = await autoDetectWallet();
    assert.equal(adapter instanceof XBullAdapter, true);
  } finally {
    FreighterAdapter.isAvailable = freighterAvailability;
    browserGlobal.window = originalWindow;
  }
});

test("autoDetectWallet throws when no supported wallet is present", async () => {
  const originalWindow = browserGlobal.window;
  const freighterAvailability = FreighterAdapter.isAvailable;

  browserGlobal.window = {};
  FreighterAdapter.isAvailable = async () => false;

  try {
    await assert.rejects(() => autoDetectWallet(), WalletDetectionError);
  } finally {
    FreighterAdapter.isAvailable = freighterAvailability;
    browserGlobal.window = originalWindow;
  }
});
