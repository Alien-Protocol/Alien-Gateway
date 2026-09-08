"use client";

import { type ReactNode } from "react";
import { TxProvider } from "@/context/TxContext";
import { WalletProvider } from "@/context/WalletContext";

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <WalletProvider>
      <TxProvider>{children}</TxProvider>
    </WalletProvider>
  );
}
