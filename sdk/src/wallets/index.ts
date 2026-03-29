import type { WalletAdapter } from "./shared";
import { WalletDetectionError, assertBrowserEnvironment } from "./shared";

export async function autoDetectWallet(): Promise<WalletAdapter> {
  assertBrowserEnvironment();

  const { FreighterAdapter } = await import("./freighter");
  if (await FreighterAdapter.isAvailable()) {
    return new FreighterAdapter();
  }

  const { XBullAdapter } = await import("./xbull");
  if (XBullAdapter.isAvailable()) {
    return new XBullAdapter();
  }

  throw new WalletDetectionError("No supported Stellar wallet found. Install Freighter or xBull.");
}

export { FreighterAdapter } from "./freighter";
export { XBullAdapter } from "./xbull";
export type { WalletAdapter } from "./shared";
export { WalletDetectionError, assertBrowserEnvironment } from "./shared";
