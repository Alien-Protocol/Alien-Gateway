"use client";

import { AssetIcon } from "@/components/app/AssetIcon";
import { Button } from "@/components/ui/button";
import { formatUsd } from "@/lib/format";
import { cn } from "@/lib/utils";

export function AmountInput({
  value,
  onChange,
  max,
  usdPrice,
  disabled,
  symbol,
}: {
  value: string;
  onChange: (v: string) => void;
  max?: number;
  usdPrice?: number;
  disabled?: boolean;
  symbol: string;
}) {
  const n = Number(value.replace(/,/g, "")) || 0;
  const usd = usdPrice != null ? n * usdPrice : undefined;
  return (
    <div
      className={cn(
        "border border-white/25 bg-black p-3",
        disabled && "opacity-50",
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <input
          inputMode="decimal"
          disabled={disabled}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="0.00"
          className="w-full bg-transparent font-exo text-2xl font-semibold tabular-nums text-white outline-none placeholder:text-white/25"
        />
        <span className="inline-flex shrink-0 items-center gap-2 border border-white/20 bg-white/5 px-2.5 py-1">
          <AssetIcon symbol={symbol} size={22} />
          <span className="font-orbitron text-[12px] font-semibold uppercase tracking-wider text-white">
            {symbol}
          </span>
        </span>
      </div>
      <div className="mt-2 flex items-center justify-between font-exo text-sm text-white/50">
        <span className="tabular-nums">
          {usd != null ? formatUsd(usd) : "—"}
        </span>
        <span className="flex items-center gap-2">
          {max != null ? (
            <>
              <span className="tabular-nums">Bal {max.toLocaleString()}</span>
              <Button
                type="button"
                variant="outline"
                size="xs"
                disabled={disabled}
                onClick={() => onChange(String(max))}
              >
                Max
              </Button>
            </>
          ) : null}
        </span>
      </div>
    </div>
  );
}
