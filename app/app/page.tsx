"use client";

import { HealthFactorGauge } from "@/components/app/HealthFactorGauge";
import { GlassCard, MetricCard } from "@/components/app/MetricCard";
import { AssetIcon } from "@/components/app/AssetIcon";
import { ConnectWalletButton } from "@/components/app/ConnectWalletButton";
import { EmptyState } from "@/components/app/EmptyState";
import { UtilizationBar } from "@/components/app/UtilizationBar";
import { PageHeader } from "@/components/app/PageHeader";
import { Button } from "@/components/ui/button";
import { useWallet } from "@/context/WalletContext";
import { useProtocolState } from "@/hooks/useProtocol";
import { formatAmount, formatUsd } from "@/lib/format";
import { RelativeTime } from "@/components/app/RelativeTime";
import { ADDRESSES, BORROW_APR_BPS } from "@/lib/protocol/constants";
import { borrowLimitRemaining } from "@/lib/protocol/math";
import { derivePosition } from "@/lib/protocol/selectors";
import Link from "next/link";
import type { ReactNode } from "react";

export default function DashboardPage() {
  const { isConnected, address } = useWallet();
  const state = useProtocolState();
  const user = address ?? ADDRESSES.you;
  const pos = derivePosition(user, state);
  const debt = state.debts[user];
  const supply = state.supplies[user] ?? 0;
  const limit = borrowLimitRemaining(
    pos.collateral,
    state.assets,
    debt?.total ?? 0,
  );
  const events = state.events.filter((e) => !e.user || e.user === user);

  return (
    <div className="space-y-6">
      <PageHeader
        kicker="Portfolio"
        title="Your position"
        description="Deposit RWA collateral, borrow USDC, or earn by supplying liquidity."
      />

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
        <MetricCard
          label="Collateral value"
          value={isConnected ? formatUsd(pos.collateralValueUsd) : "—"}
          hint="Marked to oracle"
        />
        <MetricCard
          label="Total debt"
          value={isConnected ? formatUsd(debt?.total ?? 0) : "—"}
          hint="Principal + interest"
        />
        <GlassCard>
          {isConnected ? (
            <HealthFactorGauge hf={pos.healthFactor} />
          ) : (
            <>
              <p className="font-raj text-[11px] font-semibold uppercase tracking-[0.18em] text-white/45">
                Health factor
              </p>
              <p className="mt-2 text-xl text-white/30">Connect wallet</p>
            </>
          )}
        </GlassCard>
        <MetricCard
          label="Borrow available"
          value={isConnected ? formatUsd(limit) : "—"}
          hint="Against your vault"
        />
        <MetricCard
          label="Your supply"
          value={isConnected ? formatUsd(supply) : "—"}
          hint="USDC earning yield"
        />
        <GlassCard>
          <UtilizationBar bps={state.pool.utilizationBps} />
          <p className="mt-3 text-xs text-white/40">
            Borrow APR {(BORROW_APR_BPS / 100).toFixed(2)}%
          </p>
        </GlassCard>
      </div>

      {!isConnected ? (
        <EmptyState
          title="Connect your wallet"
          body="Connect to view collateral, debt, and supply — then deposit, lend, or borrow."
          action={<ConnectWalletButton />}
        />
      ) : (
        <>
          <div className="flex flex-wrap gap-2">
            <Button asChild>
              <Link href="/app/vault">Manage vault</Link>
            </Button>
            <Button asChild variant="outline">
              <Link href="/app/borrow">Borrow / repay</Link>
            </Button>
            <Button asChild variant="outline">
              <Link href="/app/lend">Lend USDC</Link>
            </Button>
          </div>

          <div className="grid gap-4 lg:grid-cols-2">
            <GlassCard>
              <h2 className="font-orbitron text-base tracking-wide">
                Collateral
              </h2>
              {pos.collateral.length ? (
                <div className="mt-4 overflow-x-auto">
                  <table className="w-full min-w-[22rem] text-left text-sm">
                    <thead className="font-raj text-[11px] uppercase tracking-wider text-white/40">
                      <tr>
                        <th className="pb-2">Asset</th>
                        <th className="pb-2 text-right">Amount</th>
                        <th className="pb-2 text-right">Value</th>
                      </tr>
                    </thead>
                    <tbody>
                      {pos.collateral.map((c) => (
                        <tr key={c.symbol} className="border-t border-white/8">
                          <td className="py-2">
                            <span className="inline-flex items-center gap-2">
                              <AssetIcon symbol={c.symbol} size={28} />
                              {c.symbol}
                            </span>
                          </td>
                          <td className="py-2 text-right tabular-nums">
                            {formatAmount(c.amount)}
                          </td>
                          <td className="py-2 text-right tabular-nums">
                            {formatUsd(c.valueUsd)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="mt-4 text-sm text-white/45">
                  No collateral yet. Deposit RWAs in the vault.
                </p>
              )}
            </GlassCard>

            <GlassCard>
              <h2 className="font-orbitron text-base tracking-wide">Debt</h2>
              <dl className="mt-4 space-y-2 text-sm">
                <DebtRow k="Principal" v={formatUsd(debt?.principal ?? 0)} />
                <DebtRow
                  k="Interest"
                  v={formatUsd(debt?.accruedInterest ?? 0)}
                />
                <DebtRow k="APR" v={`${(BORROW_APR_BPS / 100).toFixed(2)}%`} />
                <DebtRow k="Total owed" v={formatUsd(debt?.total ?? 0)} />
              </dl>
            </GlassCard>
          </div>

          {events.length ? (
            <GlassCard>
              <h2 className="font-orbitron text-base tracking-wide">
                Recent activity
              </h2>
              <ul className="mt-4 divide-y divide-white/8 text-sm">
                {events.slice(0, 5).map((e) => (
                  <li
                    key={e.id}
                    className="flex flex-wrap items-center justify-between gap-2 py-2.5"
                  >
                    <span className="text-white/85">{e.type}</span>
                    <span className="text-white/45">
                      {e.asset ?? e.note ?? "—"}{" "}
                      {e.amount != null ? formatAmount(e.amount) : ""}
                    </span>
                    <span className="text-xs text-white/35">
                      <RelativeTime at={e.at} />
                    </span>
                  </li>
                ))}
              </ul>
            </GlassCard>
          ) : null}
        </>
      )}
    </div>
  );
}

function DebtRow({ k, v }: { k: string; v: ReactNode }) {
  return (
    <div className="flex justify-between gap-4">
      <dt className="text-white/45">{k}</dt>
      <dd className="tabular-nums text-white">{v}</dd>
    </div>
  );
}
