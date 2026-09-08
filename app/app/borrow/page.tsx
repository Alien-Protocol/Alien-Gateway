"use client";

import { AmountInput } from "@/components/app/AmountInput";
import { ConnectWalletButton } from "@/components/app/ConnectWalletButton";
import { EmptyState } from "@/components/app/EmptyState";
import { GlassCard, MetricCard } from "@/components/app/MetricCard";
import { PreviewPanel, hfText, usdText } from "@/components/app/PreviewPanel";
import { PageHeader } from "@/components/app/PageHeader";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTx } from "@/context/TxContext";
import { useWallet } from "@/context/WalletContext";
import { useProtocolState } from "@/hooks/useProtocol";
import { formatUsd, parseAmount } from "@/lib/format";
import { ADDRESSES, BORROW_APR_BPS } from "@/lib/protocol/constants";
import {
  borrowLimitRemaining,
  ltvPct,
  previewHfAfterBorrow,
  previewHfAfterRepay,
  priceOf,
  splitRepay,
  weightedLiqUsd,
} from "@/lib/protocol/math";
import { derivePosition } from "@/lib/protocol/selectors";
import { pool } from "@/lib/protocol";
import { useState } from "react";

export default function BorrowPage() {
  const { isConnected, address } = useWallet();
  const state = useProtocolState();
  const { execute } = useTx();
  const user = address ?? ADDRESSES.you;
  const [amount, setAmount] = useState("");
  const n = parseAmount(amount);
  const pos = derivePosition(user, state);
  const debt = state.debts[user];
  const limit = borrowLimitRemaining(pos.collateral, state.assets, debt?.total ?? 0);
  const weighted = weightedLiqUsd(pos.collateral, state.assets);
  const borrowPaused = state.pause.pool.borrow || state.pause.vault.borrow;
  const repayPaused = state.pause.pool.repay;
  const staleForUser = pos.collateral.some((c) => {
    const p = priceOf(c.symbol, state.prices);
    return p && !p.fresh;
  });
  const usdcStale = priceOf("USDC", state.prices)?.fresh === false;
  const oracleBlocked = staleForUser || usdcStale || state.oraclePaused;
  const nextBorrowHf = previewHfAfterBorrow(weighted, debt?.total ?? 0, n);
  const nextLtv = ltvPct(pos.collateralValueUsd, (debt?.total ?? 0) + n);
  const repaySplit = debt
    ? splitRepay(Math.min(n, debt.total), debt)
    : { interestPaid: 0, principalPaid: 0 };
  const nextRepayHf = previewHfAfterRepay(weighted, debt?.total ?? 0, n);

  if (!isConnected) {
    return (
      <EmptyState
        title="Connect to borrow"
        body="Borrow USDC against vault collateral. Repay applies interest first, then principal."
        action={<ConnectWalletButton />}
      />
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        kicker="Credit"
        title="Borrow / repay"
        description="Borrow USDC against your vault collateral, or repay outstanding debt."
      />

      <div className="grid gap-3 sm:grid-cols-3">
        <MetricCard label="Available to borrow" value={formatUsd(limit)} />
        <MetricCard label="Your debt" value={formatUsd(debt?.total ?? 0)} />
        <MetricCard
          label="APR"
          value={`${(BORROW_APR_BPS / 100).toFixed(2)}%`}
        />
      </div>

      <Tabs defaultValue="borrow" onValueChange={() => setAmount("")}>
        <TabsList>
          <TabsTrigger value="borrow">Borrow</TabsTrigger>
          <TabsTrigger value="repay">Repay</TabsTrigger>
        </TabsList>

        <TabsContent value="borrow" className="mt-4">
          <div className="grid gap-4 lg:grid-cols-2">
            <GlassCard className="space-y-4">
              {borrowPaused ? (
                <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                  Pool paused: borrow
                </p>
              ) : null}
              {oracleBlocked ? (
                <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                  Oracle feed stale — borrow disabled until prices are fresh.
                </p>
              ) : null}
              <AmountInput
                symbol="USDC"
                value={amount}
                onChange={setAmount}
                max={limit}
                usdPrice={1}
                disabled={borrowPaused || oracleBlocked}
              />
              <PreviewPanel
                rows={[
                  { label: "Health factor", before: hfText(pos.healthFactor), after: hfText(nextBorrowHf) },
                  {
                    label: "LTV",
                    before: `${ltvPct(pos.collateralValueUsd, debt?.total ?? 0).toFixed(1)}%`,
                    after: `${nextLtv.toFixed(1)}%`,
                  },
                  { label: "Debt", before: usdText(debt?.total ?? 0), after: usdText((debt?.total ?? 0) + n) },
                ]}
              />
              <Button
                type="button"
                disabled={borrowPaused || oracleBlocked || n <= 0 || n > limit}
                onClick={() =>
                  execute(
                    {
                      title: "Borrow",
                      detail: `Borrow ${n} USDC against vault collateral.`,
                    },
                    () => pool.borrow(user, "USDC", n),
                    `Borrowed ${n} USDC`,
                  )
                }
              >
                Borrow
              </Button>
            </GlassCard>
            <GlassCard>
              <h2 className="font-orbitron text-base tracking-wide">How it works</h2>
              <ul className="mt-3 list-disc space-y-2 pl-4 text-sm text-white/55">
                <li>Borrow limit is based on your collateral and max LTV.</li>
                <li>Interest accrues continuously at a fixed APR.</li>
                <li>Keep your health factor above 1.00 to stay safe.</li>
              </ul>
            </GlassCard>
          </div>
        </TabsContent>

        <TabsContent value="repay" className="mt-4">
          <GlassCard className="mx-auto max-w-xl space-y-4">
            {repayPaused ? (
              <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                Pool paused: repay
              </p>
            ) : null}
            <p className="text-sm text-white/55">
              Interest due {formatUsd(debt?.accruedInterest ?? 0)} · principal{" "}
              {formatUsd(debt?.principal ?? 0)}
            </p>
            <AmountInput
              symbol="USDC"
              value={amount}
              onChange={setAmount}
              max={Math.min(debt?.total ?? 0, state.wallets[user]?.USDC ?? 0)}
              usdPrice={1}
              disabled={repayPaused}
            />
            <p className="text-sm text-white/60">
              Payments apply to interest first, then principal —{" "}
              {formatUsd(repaySplit.interestPaid)} interest,{" "}
              {formatUsd(repaySplit.principalPaid)} principal.
            </p>
            <PreviewPanel
              rows={[
                { label: "Health factor", before: hfText(pos.healthFactor), after: hfText(nextRepayHf) },
                {
                  label: "Debt",
                  before: usdText(debt?.total ?? 0),
                  after: usdText(Math.max(0, (debt?.total ?? 0) - n)),
                },
              ]}
            />
            <Button
              type="button"
              disabled={repayPaused || n <= 0}
              onClick={() =>
                execute(
                  {
                    title: "Repay",
                    detail: `Repay ${n} USDC.`,
                  },
                  () => pool.repay(user, n),
                  `Repaid ${n} USDC`,
                )
              }
            >
              Repay
            </Button>
          </GlassCard>
        </TabsContent>
      </Tabs>
    </div>
  );
}
