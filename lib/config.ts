export type DataSource = "mock" | "http";
export type StellarNetwork = "testnet" | "public";

function readDataSource(): DataSource {
  const value = process.env.NEXT_PUBLIC_DATA_SOURCE?.trim().toLowerCase();
  return value === "http" ? "http" : "mock";
}

function readStellarNetwork(): StellarNetwork {
  const value = process.env.NEXT_PUBLIC_STELLAR_NETWORK?.trim().toLowerCase();
  return value === "public" || value === "mainnet" ? "public" : "testnet";
}

/**
 * Frontend runtime config. Pages never read `process.env` directly —
 * switch mock → live backend by changing env, not UI code.
 */
export const appConfig = {
  dataSource: readDataSource(),
  apiBaseUrl: process.env.NEXT_PUBLIC_API_BASE_URL?.replace(/\/$/, "") ?? "",
  explorerTxUrl:
    process.env.NEXT_PUBLIC_EXPLORER_TX_URL ??
    "https://stellar.expert/explorer/testnet/tx",
  pollIntervalMs: Number(process.env.NEXT_PUBLIC_POLL_INTERVAL_MS ?? 8_000),
  requestTimeoutMs: Number(process.env.NEXT_PUBLIC_REQUEST_TIMEOUT_MS ?? 15_000),
  stellarNetwork: readStellarNetwork(),
  /** Optional — enables WalletConnect / mobile wallets in the kit modal */
  walletConnectProjectId:
    process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID?.trim() ?? "",
} as const;
