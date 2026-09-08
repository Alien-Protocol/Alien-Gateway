"use client";

import { AmountInput } from "@/components/app/AmountInput";
import { ConnectWalletButton } from "@/components/app/ConnectWalletButton";
import { EmptyState } from "@/components/app/EmptyState";
import { GlassCard, MetricCard } from "@/components/app/MetricCard";
import { UtilizationBar } from "@/components/app/UtilizationBar";
import { PageHeader } from "@/components/app/PageHeader";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTx } from "@/context/TxContext";
import { useWallet } from "@/context/WalletContext";
import { useProtocolState } from "@/hooks/useProtocol";
import { formatUsd, parseAmount } from "@/lib/format";
import { ADDRESSES, BORROW_APR_BPS } from "@/lib/protocol/constants";
import { supplierApyPct } from "@/lib/protocol/math";
import { pool } from "@/lib/protocol";
import { useState } from "react";

export default function LendPage() {
  const { isConnected, address } = useWallet();
  const state = useProtocolState();
  const { execute } = useTx();
  const user = address ?? ADDRESSES.you;
  const [amount, setAmount] = useState("");
  const n = parseAmount(amount);
  const supplied = state.supplies[user] ?? 0;
  const wallet = state.wallets[user]?.USDC ?? 0;
  const cap = Math.min(supplied, state.pool.availableLiquidity);
  const supplyPaused = state.pause.pool.supply;
  const withdrawPaused = state.pause.pool.withdrawLiquidity;
  const apy = supplierApyPct(state.pool.utilizationBps, BORROW_APR_BPS);

  return (
    <div className="space-y-6">
      <PageHeader
        kicker="Earn"
        title="Lend USDC"
        description="Supply USDC to the pool and earn yield from borrowers."
      />

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard label="Total supply" value={formatUsd(state.pool.totalSupply)} accent="violet" />
        <MetricCard label="Total borrowed" value={formatUsd(state.pool.totalBorrowed)} accent="red" />
        <MetricCard
          label="Available liquidity"
          value={formatUsd(state.pool.availableLiquidity)}
          accent="cyan"
        />
        <GlassCard>
          <UtilizationBar bps={state.pool.utilizationBps} />
        </GlassCard>
      </div>

      {!isConnected ? (
        <EmptyState
          title="Connect to supply"
          body="Provide USDC to the pool. Withdrawals are capped by your supply and available liquidity."
          action={<ConnectWalletButton />}
        />
      ) : (
        <div className="grid gap-4 lg:grid-cols-2">
          <GlassCard className="space-y-4">
            <Tabs defaultValue="supply" onValueChange={() => setAmount("")}>
              <TabsList>
                <TabsTrigger value="supply">Supply</TabsTrigger>
                <TabsTrigger value="withdraw">Withdraw liquidity</TabsTrigger>
              </TabsList>
              <TabsContent value="supply" className="mt-4 space-y-4">
                {supplyPaused ? (
                  <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                    Pool paused: supply
                  </p>
                ) : null}
                <AmountInput
                  symbol="USDC"
                  value={amount}
                  onChange={setAmount}
                  max={wallet}
                  usdPrice={1}
                  disabled={supplyPaused}
                />
                <Button
                  type="button"
                  disabled={supplyPaused || n <= 0 || n > wallet}
                  onClick={() =>
                    execute(
                      {
                        title: "Supply",
                        detail: `Supply ${n} USDC to the lending pool.`,
                      },
                      () => pool.supply(user, n),
                      `Supplied ${n} USDC`,
                    )
                  }
                >
                  Supply
                </Button>
              </TabsContent>
              <TabsContent value="withdraw" className="mt-4 space-y-4">
                {withdrawPaused ? (
                  <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                    Pool paused: withdrawLiquidity
                  </p>
                ) : null}
                <AmountInput
                  symbol="USDC"
                  value={amount}
                  onChange={setAmount}
                  max={cap}
                  usdPrice={1}
                  disabled={withdrawPaused}
                />
                <p className="text-xs text-white/40">
                  Cap is min(your supply {formatUsd(supplied)}, available{" "}
                  {formatUsd(state.pool.availableLiquidity)}).
                </p>
                <Button
                  type="button"
                  disabled={withdrawPaused || n <= 0 || n > cap}
                  onClick={() =>
                    execute(
                      {
                        title: "Withdraw liquidity",
                        detail: `Withdraw ${n} USDC from pool liquidity.`,
                      },
                      () => pool.withdrawLiquidity(user, n),
                      `Withdrew ${n} USDC liquidity`,
                    )
                  }
                >
                  Withdraw liquidity
                </Button>
              </TabsContent>
            </Tabs>
          </GlassCard>

          <GlassCard>
            <h2 className="font-orbitron text-base tracking-wide">Your supply</h2>
            <p className="mt-3 font-sans text-3xl font-semibold tabular-nums">
              {formatUsd(supplied)}
            </p>
            <p className="mt-4 text-sm leading-relaxed text-white/50">
              Estimated supplier APY {apy.toFixed(2)}% · borrower APR{" "}
              {(BORROW_APR_BPS / 100).toFixed(2)}%.
            </p>
          </GlassCard>
        </div>
      )}
    </div>
  );
}
