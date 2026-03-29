import type { WalletAdapter } from "./shared";
export declare function autoDetectWallet(): Promise<WalletAdapter>;
export { FreighterAdapter } from "./freighter";
export { XBullAdapter } from "./xbull";
export type { WalletAdapter } from "./shared";
export { WalletDetectionError, assertBrowserEnvironment } from "./shared";
