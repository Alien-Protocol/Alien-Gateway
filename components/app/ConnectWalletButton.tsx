"use client";

import { AssetIcon } from "@/components/app/AssetIcon";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useWallet } from "@/context/WalletContext";
import { useProtocolState } from "@/hooks/useProtocol";
import { formatAmount, truncateAddress } from "@/lib/format";
import { cn } from "@/lib/utils";

export function ConnectWalletButton({
  compact = false,
}: {
  compact?: boolean;
}) {
  const {
    status,
    address,
    walletId,
    connect,
    disconnect,
    openProfile,
    isConnected,
  } = useWallet();
  const { wallets } = useProtocolState();
  const bal = address ? (wallets[address] ?? {}) : {};

  if (!isConnected || !address) {
    return (
      <Button
        type="button"
        size={compact ? "sm" : "default"}
        className={cn(compact && "h-10 px-3 text-[10px] sm:px-4 sm:text-xs")}
        onClick={() => void connect()}
        disabled={status === "connecting"}
      >
        {status === "connecting" ? (
          "…"
        ) : (
          <>
            <span className="sm:hidden">Connect</span>
            <span className="hidden sm:inline">Connect Wallet</span>
          </>
        )}
      </Button>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          type="button"
          variant="outline"
          size={compact ? "sm" : "default"}
          className={cn(compact && "h-10 px-2.5 sm:px-4")}
        >
          <span className="font-orbitron text-[10px] tracking-[0.12em] text-white sm:text-[11px] sm:tracking-[0.14em]">
            {truncateAddress(address, 3, 3)}
          </span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-[min(18rem,calc(100vw-1.5rem))] p-4"
      >
        <DropdownMenuLabel className="p-0 font-normal">
          {walletId ? (
            <p className="mb-1 font-orbitron text-[10px] uppercase tracking-[0.16em] text-white/45">
              {walletId}
            </p>
          ) : null}
          <p className="break-all font-exo text-xs leading-relaxed text-white/70">
            {address}
          </p>
        </DropdownMenuLabel>
        <DropdownMenuSeparator className="my-3 bg-white/15" />
        <div className="space-y-2 font-exo text-sm tabular-nums text-white/80">
          <p className="flex items-center gap-2">
            <AssetIcon symbol="USDC" size={20} /> {formatAmount(bal.USDC ?? 0)}{" "}
            USDC
          </p>
          <p className="flex items-center gap-2">
            <AssetIcon symbol="XLM" size={20} /> {formatAmount(bal.XLM ?? 0)} XLM
          </p>
          <p className="flex items-center gap-2">
            <AssetIcon symbol="tBILL" size={20} />{" "}
            {formatAmount(bal.tBILL ?? 0)} tBILL
          </p>
        </div>
        <div className="mt-4 flex flex-col gap-2">
          <Button
            type="button"
            variant="outline"
            className="w-full"
            onClick={() => void openProfile()}
          >
            Wallet profile
          </Button>
          <Button
            type="button"
            variant="outline"
            className="w-full"
            onClick={() => void disconnect()}
          >
            Disconnect
          </Button>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
