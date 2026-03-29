import type { WalletAdapter } from "./shared";
import { WalletDetectionError, getBrowserWindow } from "./shared";

export interface XBullProvider {
  connect?(): Promise<void>;
  requestAccess?(): Promise<void>;
  getPublicKey?(): Promise<string>;
  getAddress?(): Promise<string>;
  signTransaction?(xdr: string, options?: Record<string, unknown>): Promise<string | { signedXdr?: string; signedTxXdr?: string; xdr?: string }>;
  sign?(xdr: string, options?: Record<string, unknown>): Promise<string | { signedXdr?: string; signedTxXdr?: string; xdr?: string }>;
}

interface XBullWindow {
  xBullSDK?: XBullProvider;
  xBull?: XBullProvider;
}

type BrowserGlobal = typeof globalThis & { window?: unknown };

export class XBullAdapter implements WalletAdapter {
  private publicKey?: string;

  public constructor(private readonly provider?: XBullProvider) {}

  public static isAvailable(): boolean {
    if (typeof (globalThis as BrowserGlobal).window === "undefined") {
      return false;
    }

    const browserWindow = getBrowserWindow<XBullWindow>();
    return Boolean(browserWindow.xBullSDK ?? browserWindow.xBull);
  }

  public async connect(): Promise<void> {
    const provider = this.getProvider();

    if (provider.connect) {
      await provider.connect();
    } else if (provider.requestAccess) {
      await provider.requestAccess();
    }

    this.publicKey = await this.getPublicKey();
  }

  public async getPublicKey(): Promise<string> {
    if (this.publicKey) {
      return this.publicKey;
    }

    const provider = this.getProvider();
    const publicKey = provider.getPublicKey
      ? await provider.getPublicKey()
      : provider.getAddress
        ? await provider.getAddress()
        : undefined;

    if (!publicKey) {
      throw new WalletDetectionError("xBull did not expose an active public key.");
    }

    this.publicKey = publicKey;
    return publicKey;
  }

  public async signTransaction(xdr: string): Promise<string> {
    const provider = this.getProvider();
    const signer = provider.signTransaction ?? provider.sign;

    if (!signer) {
      throw new WalletDetectionError("xBull does not support transaction signing.");
    }

    const signed = await signer.call(provider, xdr);

    if (typeof signed === "string") {
      return signed;
    }

    const signedXdr = signed.signedXdr ?? signed.signedTxXdr ?? signed.xdr;
    if (!signedXdr) {
      throw new WalletDetectionError("xBull returned an invalid signed transaction payload.");
    }

    return signedXdr;
  }

  private getProvider(): XBullProvider {
    if (this.provider) {
      return this.provider;
    }

    const browserWindow = getBrowserWindow<XBullWindow>();
    const provider = browserWindow.xBullSDK ?? browserWindow.xBull;
    if (!provider) {
      throw new WalletDetectionError("xBull is not available in this browser.");
    }

    return provider;
  }
}
