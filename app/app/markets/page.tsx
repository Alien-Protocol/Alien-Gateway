"use client";

import { AssetIcon } from "@/components/app/AssetIcon";
import { BpsBadge } from "@/components/app/BpsBadge";
import { GlassCard } from "@/components/app/MetricCard";
import { PageHeader } from "@/components/app/PageHeader";
import { RelativeTime } from "@/components/app/RelativeTime";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useProtocolState } from "@/hooks/useProtocol";
import { cn } from "@/lib/utils";
import { formatUsd } from "@/lib/format";
import { priceOf } from "@/lib/protocol/math";
import type { AssetConfig } from "@/lib/protocol/types";
import Link from "next/link";
import { useState, type ReactNode } from "react";

export default function MarketsPage() {
  const state = useProtocolState();
  const [open, setOpen] = useState<string | null>(null);
  const selected = state.assets.find((a) => a.symbol === open);

  return (
    <div className="space-y-6">
      <PageHeader
        kicker="Assets"
        title="Markets"
        description="Supported collateral and USDC liquidity on Alien Protocol."
      />

      <GlassCard padding={false}>
        <div className="overflow-x-auto">
          <Table className="min-w-[36rem] text-sm sm:min-w-[56rem] sm:text-base">
            <TableHeader>
              <TableRow className="border-white/10 hover:bg-transparent">
                <TableHead className="px-5 py-4 font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Asset
                </TableHead>
                <TableHead className="px-3 py-4 font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Oracle
                </TableHead>
                <TableHead className="px-3 py-4 font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Max LTV
                </TableHead>
                <TableHead className="px-3 py-4 font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Liq. threshold
                </TableHead>
                <TableHead className="px-3 py-4 text-right font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Posted / supplied
                </TableHead>
                <TableHead className="px-5 py-4 text-right font-raj text-[13px] uppercase tracking-wider text-white/45">
                  Utilization
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {state.assets.map((a) => {
                const px = priceOf(a.symbol, state.prices);
                const isUsdc = a.symbol === "USDC";
                return (
                  <TableRow
                    key={a.symbol}
                    className="cursor-pointer border-white/6 hover:bg-white/[0.04]"
                    onClick={() => setOpen(a.symbol)}
                  >
                    <TableCell className="px-5 py-4">
                      <span className="inline-flex items-center gap-3">
                        <AssetIcon symbol={a.symbol} size={40} />
                        <span>
                          <span className="block text-[15px] font-semibold">
                            {a.symbol}
                          </span>
                          <span className="text-sm text-white/45">{a.name}</span>
                        </span>
                      </span>
                    </TableCell>
                    <TableCell className="px-3 py-4">
                      <div className="text-[15px] tabular-nums">
                        {formatUsd(px?.price ?? 0, {
                          digits:
                            a.symbol === "tBILL" ||
                            a.symbol === "USDC" ||
                            a.symbol === "tINV"
                              ? 3
                              : 2,
                        })}
                      </div>
                      <div className="mt-1 flex items-center gap-2 text-sm">
                        <span className="text-white/40">
                          {px ? <RelativeTime at={px.timestamp} /> : "—"}
                        </span>
                        <FreshBadge fresh={Boolean(px?.fresh)} />
                      </div>
                    </TableCell>
                    <TableCell className="px-3 py-3">
                      {isUsdc ? (
                        <span className="text-white/35">n/a</span>
                      ) : (
                        <BpsBadge bps={a.maxLtvBps} />
                      )}
                    </TableCell>
                    <TableCell className="px-3 py-4">
                      {isUsdc ? (
                        <span className="text-white/35">n/a</span>
                      ) : (
                        <BpsBadge bps={a.liquidationThresholdBps} />
                      )}
                    </TableCell>
                    <TableCell className="px-3 py-4 text-right tabular-nums">
                      {isUsdc
                        ? formatUsd(state.pool.totalSupply)
                        : formatUsd(
                            state.analytics.collateralPosted[a.symbol] ?? 0,
                          )}
                    </TableCell>
                    <TableCell className="px-5 py-4 text-right tabular-nums">
                      {isUsdc
                        ? `${(state.pool.utilizationBps / 100).toFixed(2)}%`
                        : "—"}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
      </GlassCard>

      <AssetDrawer
        asset={selected ?? null}
        open={Boolean(selected)}
        onClose={() => setOpen(null)}
      />
    </div>
  );
}

function FreshBadge({ fresh }: { fresh: boolean }) {
  return (
    <Badge
      className={cn(
        "rounded-none px-2.5 py-0.5 font-orbitron text-[10px] font-semibold uppercase tracking-wider",
        fresh
          ? "border-white bg-white text-black hover:bg-white"
          : "border-white/40 bg-transparent text-white/50",
      )}
      variant="outline"
    >
      {fresh ? "Fresh" : "Stale"}
    </Badge>
  );
}

function AssetDrawer({
  asset,
  open,
  onClose,
}: {
  asset: AssetConfig | null;
  open: boolean;
  onClose: () => void;
}) {
  const state = useProtocolState();
  if (!asset) return null;

  const px = priceOf(asset.symbol, state.prices);

  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()}>
      <SheetContent
        side="right"
        className="w-full max-w-md border-white/15 bg-black p-6"
        showCloseButton={false}
      >
        <SheetHeader className="p-0 text-left">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-3">
              <AssetIcon symbol={asset.symbol} size={44} />
              <div>
                <SheetTitle className="font-orbitron text-lg">
                  {asset.symbol}
                </SheetTitle>
                <p className="text-sm text-white/50">{asset.name}</p>
              </div>
            </div>
            <Button type="button" variant="outline" onClick={onClose}>
              Close
            </Button>
          </div>
        </SheetHeader>

        <dl className="mt-6 space-y-2 text-sm">
          <Row k="Type" v={asset.type} />
          <Row
            k="Max LTV"
            v={
              asset.symbol === "USDC"
                ? "n/a"
                : `${(asset.maxLtvBps / 100).toFixed(0)}%`
            }
          />
          <Row
            k="Liquidation threshold"
            v={
              asset.symbol === "USDC"
                ? "n/a"
                : `${(asset.liquidationThresholdBps / 100).toFixed(0)}%`
            }
          />
          <Row k="Price" v={formatUsd(px?.price ?? 0, { digits: 3 })} />
          <Row k="Oracle" v={px?.fresh ? "Live" : "Stale"} />
        </dl>

        <div className="mt-6 flex flex-col gap-2">
          {asset.symbol !== "USDC" ? (
            <Button asChild>
              <Link href="/app/vault">Use as collateral</Link>
            </Button>
          ) : (
            <>
              <Button asChild>
                <Link href="/app/lend">Supply</Link>
              </Button>
              <Button asChild variant="outline">
                <Link href="/app/borrow">Borrow</Link>
              </Button>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}

function Row({ k, v }: { k: string; v: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-white/6 py-2">
      <dt className="text-white/45">{k}</dt>
      <dd className="text-right text-white">{v}</dd>
    </div>
  );
}
