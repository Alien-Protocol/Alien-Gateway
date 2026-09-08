import {
  StellarWalletsKit,
  Networks,
  SwkAppDarkTheme,
  KitEventType,
  type ModuleInterface,
} from "@creit.tech/stellar-wallets-kit";
import { defaultModules } from "@creit.tech/stellar-wallets-kit/modules/utils";
import { appConfig } from "@/lib/config";

let initialized = false;

const ALIEN_THEME = {
  ...SwkAppDarkTheme,
  background: "#000000",
  "background-secondary": "#0a0a0a",
  "foreground-strong": "#ffffff",
  foreground: "rgba(255,255,255,0.85)",
  "foreground-secondary": "rgba(255,255,255,0.55)",
  primary: "#ffffff",
  "primary-foreground": "#000000",
  border: "rgba(255,255,255,0.22)",
  "border-radius": "0px",
  "font-family": "var(--font-exo), ui-sans-serif, system-ui, sans-serif",
};

function resolveNetwork(): Networks {
  return appConfig.stellarNetwork === "public"
    ? Networks.PUBLIC
    : Networks.TESTNET;
}

async function buildModules(): Promise<ModuleInterface[]> {
  const modules: ModuleInterface[] = [...defaultModules()];

  const projectId = appConfig.walletConnectProjectId;
  if (projectId) {
    const { WalletConnectModule, WalletConnectTargetChain } = await import(
      "@creit.tech/stellar-wallets-kit/modules/wallet-connect"
    );
    modules.push(
      new WalletConnectModule({
        projectId,
        allowedChains: [
          appConfig.stellarNetwork === "public"
            ? WalletConnectTargetChain.PUBLIC
            : WalletConnectTargetChain.TESTNET,
        ],
        metadata: {
          name: "Alien Protocol",
          description: "RWA lending infrastructure on Stellar + Soroban",
          url:
            typeof window !== "undefined"
              ? window.location.origin
              : "https://alienprotocol.app",
          icons: [
            typeof window !== "undefined"
              ? `${window.location.origin}/Alien-Protocol_2.png`
              : "https://alienprotocol.app/Alien-Protocol_2.png",
          ],
        },
      }),
    );
  }

  return modules;
}

/** Idempotent client-side kit bootstrap. */
export async function ensureStellarKit(): Promise<typeof StellarWalletsKit> {
  if (typeof window === "undefined") {
    throw new Error("Stellar Wallets Kit is browser-only");
  }
  if (!initialized) {
    const modules = await buildModules();
    StellarWalletsKit.init({
      modules,
      network: resolveNetwork(),
      theme: ALIEN_THEME,
      authModal: {
        hideUnsupportedWallets: false,
        showInstallLabel: true,
      },
    });
    initialized = true;
  }
  return StellarWalletsKit;
}

export { StellarWalletsKit, KitEventType };
