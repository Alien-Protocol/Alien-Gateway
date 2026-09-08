"use client";

import type { ComponentType, ReactNode } from "react";
import { TokenEURC, TokenUSDC, TokenXLM } from "@web3icons/react";
import { TbillLogo, TinvLogo, TreitLogo } from "@/components/app/TokenLogos";
import { cn } from "@/lib/utils";

type Web3Icon = ComponentType<{
  size?: number | string;
  variant?: "mono" | "branded" | "background";
  className?: string;
}>;

type Web3Entry = {
  Icon: Web3Icon;
  /** XLM branded mark is black — use background on dark UI. */
  variant: "mono" | "branded" | "background";
};

const WEB3_ICONS: Record<string, Web3Entry> = {
  USDC: { Icon: TokenUSDC, variant: "branded" },
  XLM: { Icon: TokenXLM, variant: "background" },
  EURC: { Icon: TokenEURC, variant: "branded" },
};

const RWA_FALLBACK: Record<string, (size: number) => ReactNode> = {
  tBILL: (s) => <TbillLogo size={s} />,
  tREIT: (s) => <TreitLogo size={s} />,
  tINV: (s) => <TinvLogo size={s} />,
};

export function AssetIcon({
  symbol,
  size = 36,
}: {
  symbol: string;
  size?: number;
}) {
  const web3 = WEB3_ICONS[symbol];
  const rwa = RWA_FALLBACK[symbol];

  return (
    <span
      className={cn(
        "inline-grid shrink-0 place-items-center overflow-hidden rounded-full ring-1 ring-white/30",
      )}
      style={{ width: size, height: size }}
      title={symbol}
    >
      {web3 ? (
        <web3.Icon
          size={size}
          variant={web3.variant}
          className="h-full w-full"
        />
      ) : rwa ? (
        rwa(size)
      ) : (
        <span className="grid h-full w-full place-items-center bg-white font-sans text-[11px] font-bold text-black">
          {symbol.slice(0, 2)}
        </span>
      )}
    </span>
  );
}
