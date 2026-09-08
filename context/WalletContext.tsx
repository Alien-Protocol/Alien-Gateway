"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { toast } from "sonner";
import {
  ensureStellarKit,
  KitEventType,
  StellarWalletsKit,
} from "@/lib/stellar/kit";
import { adoptConnectedUser } from "@/lib/protocol/mock/store";
import { protocol } from "@/lib/protocol";

type WalletStatus = "disconnected" | "connecting" | "connected";

type WalletContextValue = {
  status: WalletStatus;
  address: string | null;
  walletId: string | null;
  isConnected: boolean;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  openProfile: () => Promise<void>;
};

const WalletContext = createContext<WalletContextValue | null>(null);

function applyConnected(address: string) {
  adoptConnectedUser(address);
}

export function WalletProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<WalletStatus>("disconnected");
  const [address, setAddress] = useState<string | null>(null);
  const [walletId, setWalletId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    let offState = () => {};
    let offWallet = () => {};
    let offDisconnect = () => {};

    void (async () => {
      try {
        await ensureStellarKit();
        if (cancelled) return;

        offState = StellarWalletsKit.on(KitEventType.STATE_UPDATED, (event) => {
          const next = event.payload.address ?? null;
          if (next) {
            applyConnected(next);
            setAddress(next);
            setStatus("connected");
          } else {
            setAddress(null);
            setStatus("disconnected");
          }
        });

        offWallet = StellarWalletsKit.on(
          KitEventType.WALLET_SELECTED,
          (event) => {
            setWalletId(event.payload.id ?? null);
          },
        );

        offDisconnect = StellarWalletsKit.on(KitEventType.DISCONNECT, () => {
          setAddress(null);
          setWalletId(null);
          setStatus("disconnected");
        });

        try {
          const { address: restored } = await StellarWalletsKit.getAddress();
          if (!cancelled && restored) {
            applyConnected(restored);
            setAddress(restored);
            setStatus("connected");
          }
        } catch {
          // No prior session
        }
      } catch (err) {
        console.error("[wallet] kit init failed", err);
      }
    })();

    return () => {
      cancelled = true;
      offState();
      offWallet();
      offDisconnect();
    };
  }, []);

  const connect = useCallback(async () => {
    setStatus("connecting");
    try {
      await ensureStellarKit();
      const { address: next } = await StellarWalletsKit.authModal();
      applyConnected(next);
      setAddress(next);
      setStatus("connected");
      toast.success("Wallet connected");
    } catch (err) {
      setStatus(address ? "connected" : "disconnected");
      const message =
        err instanceof Error ? err.message : "Wallet connection cancelled";
      if (!/cancel|abort|dismiss/i.test(message)) {
        toast.error(message);
      }
    }
  }, [address]);

  const disconnect = useCallback(async () => {
    try {
      await ensureStellarKit();
      await StellarWalletsKit.disconnect();
    } catch {
      // still clear local session
    }
    setAddress(null);
    setWalletId(null);
    setStatus("disconnected");
    protocol.reset();
  }, []);

  const openProfile = useCallback(async () => {
    if (!address) return;
    try {
      await ensureStellarKit();
      await StellarWalletsKit.profileModal();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Could not open wallet profile";
      toast.error(message);
    }
  }, [address]);

  const value = useMemo(
    () => ({
      status,
      address,
      walletId,
      isConnected: status === "connected" && Boolean(address),
      connect,
      disconnect,
      openProfile,
    }),
    [status, address, walletId, connect, disconnect, openProfile],
  );

  return (
    <WalletContext.Provider value={value}>{children}</WalletContext.Provider>
  );
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
