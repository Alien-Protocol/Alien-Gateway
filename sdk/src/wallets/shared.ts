export interface WalletAdapter {
  connect(): Promise<void>;
  getPublicKey(): Promise<string>;
  signTransaction(xdr: string): Promise<string>;
}

export class WalletDetectionError extends Error {
  public constructor(message: string) {
    super(message);
    this.name = "WalletDetectionError";
  }
}

type BrowserGlobal = typeof globalThis & { window?: unknown };

export function assertBrowserEnvironment(): void {
  if (typeof (globalThis as BrowserGlobal).window === "undefined") {
    throw new WalletDetectionError("Wallet adapters are only available in browser environments.");
  }
}

export function getBrowserWindow<T extends object>(): T {
  assertBrowserEnvironment();
  return (globalThis as BrowserGlobal).window as T;
}
