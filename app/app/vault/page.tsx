"use client";

import { AmountInput } from "@/components/app/AmountInput";
import { AssetIcon } from "@/components/app/AssetIcon";
import { EmptyState } from "@/components/app/EmptyState";
import { GlassCard } from "@/components/app/MetricCard";
import { HealthFactorGauge } from "@/components/app/HealthFactorGauge";
import { ConnectWalletButton } from "@/components/app/ConnectWalletButton";
import { PageHeader } from "@/components/app/PageHeader";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTx } from "@/context/TxContext";
import { useWallet } from "@/context/WalletContext";
import { useProtocolState } from "@/hooks/useProtocol";
import { formatAmount, formatUsd, parseAmount } from "@/lib/format";
import { ADDRESSES, MIN_COLLATERAL_RATIO } from "@/lib/protocol/constants";
import { isWithdrawalSafeCalc, priceOf } from "@/lib/protocol/math";
import { derivePosition } from "@/lib/protocol/selectors";
import { vault } from "@/lib/protocol";
import { useMemo, useState } from "react";

export default function VaultPage() {
  const { isConnected, address } = useWallet();
  const state = useProtocolState();
  const { execute } = useTx();
  const user = address ?? ADDRESSES.you;
  const supported = state.assets.filter(
    (a) => a.supported && a.symbol !== "USDC",
  );
  const [symbol, setSymbol] = useState(supported[0]?.symbol ?? "tBILL");
  const [amount, setAmount] = useState("");
  const pos = derivePosition(user, state);
  const px = priceOf(symbol, state.prices)?.price ?? 0;
  const n = parseAmount(amount);
  const walletBal = state.wallets[user]?.[symbol] ?? 0;
  const held = state.holdings[user]?.[symbol] ?? 0;
  const debt = state.debts[user]?.total ?? 0;
  const depositPaused = state.pause.vault.deposit;
  const withdrawPaused = state.pause.vault.withdraw;

  const safe = useMemo(
    () =>
      isWithdrawalSafeCalc(
        state.holdings[user] ?? {},
        symbol,
        n,
        debt,
        state.prices,
        state.assets,
      ),
    [state, user, symbol, n, debt],
  );

  if (!isConnected) {
    return (
      <EmptyState
        title="Connect to manage the vault"
        body="Deposit tokenized RWA collateral or withdraw against your position."
        action={<ConnectWalletButton />}
      />
    );
  }

  return (
    <div className="grid gap-4 lg:grid-cols-[1fr_20rem]">
      <div className="space-y-4">
        <PageHeader
          kicker="Collateral"
          title="Vault"
          description="Deposit tokenized RWAs as collateral, or withdraw when your position allows."
        />

        <Tabs defaultValue="deposit" onValueChange={() => setAmount("")}>
          <TabsList>
            <TabsTrigger value="deposit">Deposit</TabsTrigger>
            <TabsTrigger value="withdraw">Withdraw</TabsTrigger>
          </TabsList>

          <GlassCard className="mt-4 space-y-4">
            <label className="block text-sm text-white/50">
              Asset
              <select
                value={symbol}
                onChange={(e) => {
                  setSymbol(e.target.value);
                  setAmount("");
                }}
                className="mt-1 w-full border border-white/25 bg-black px-3 py-2 font-exo text-white outline-none"
              >
                {supported.map((a) => (
                  <option key={a.symbol} value={a.symbol}>
                    {a.symbol} — {a.name}
                  </option>
                ))}
              </select>
            </label>

            <TabsContent value="deposit" className="mt-0 space-y-4">
              {depositPaused ? (
                <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                  Vault paused: deposit
                </p>
              ) : null}
              <AmountInput
                symbol={symbol}
                value={amount}
                onChange={setAmount}
                max={walletBal}
                usdPrice={px}
                disabled={depositPaused}
              />
              <p className="text-xs text-white/40">
                USD preview {formatUsd(n * px)} from mock oracle.
              </p>
              <Button
                type="button"
                disabled={depositPaused || n <= 0 || n > walletBal}
                onClick={() =>
                  execute(
                    {
                      title: "Deposit",
                      detail: `Deposit ${n} ${symbol} into the collateral vault.`,
                    },
                    () => vault.deposit(user, symbol, n),
                    `Deposited ${n} ${symbol}`,
                  )
                }
              >
                Deposit
              </Button>
            </TabsContent>

            <TabsContent value="withdraw" className="mt-0 space-y-4">
              {withdrawPaused ? (
                <p className="border border-white/40 bg-white/[0.04] px-3 py-2 font-exo text-sm text-white">
                  Vault paused: withdraw
                </p>
              ) : null}
              <p className="text-xs text-white/45">
                Posted {formatAmount(held)} {symbol}
              </p>
              <AmountInput
                symbol={symbol}
                value={amount}
                onChange={setAmount}
                max={held}
                usdPrice={px}
                disabled={withdrawPaused}
              />
              {!safe && n > 0 ? (
                <p className="border border-white/50 bg-white/[0.06] px-3 py-2 font-exo text-sm text-white">
                  Remaining collateral would fall below 110% of debt (
                  {MIN_COLLATERAL_RATIO * 100}% min ratio). Withdraw is blocked.
                </p>
              ) : null}
              <Button
                type="button"
                disabled={withdrawPaused || n <= 0 || n > held || !safe}
                onClick={() =>
                  execute(
                    {
                      title: "Withdraw",
                      detail: `Withdraw ${n} ${symbol} from the collateral vault.`,
                    },
                    () => vault.withdraw(user, symbol, n),
                    `Withdrew ${n} ${symbol}`,
                  )
                }
              >
                Withdraw
              </Button>
            </TabsContent>
          </GlassCard>
        </Tabs>
      </div>

      <GlassCard className="h-fit space-y-4 lg:sticky lg:top-28">
        <h2 className="font-orbitron text-base tracking-wide">Position summary</h2>
        <HealthFactorGauge hf={pos.healthFactor} size={112} />
        <p className="text-sm text-white/50">
          Collateral {formatUsd(pos.collateralValueUsd)}
        </p>
        <ul className="space-y-2 text-sm">
          {pos.collateral.length ? (
            pos.collateral.map((c) => (
              <li key={c.symbol} className="flex items-center justify-between">
                <span className="inline-flex items-center gap-2">
                  <AssetIcon symbol={c.symbol} size={32} />
                  {c.symbol}
                </span>
                <span className="tabular-nums">
                  {formatAmount(c.amount)} · {formatUsd(c.valueUsd)}
                </span>
              </li>
            ))
          ) : (
            <li className="text-white/40">No collateral posted.</li>
          )}
        </ul>
        <p className="text-xs text-white/35">
          Max LTV {(pos.maxLtvBps / 100).toFixed(0)}% · liquidation at{" "}
          {(pos.liquidationThresholdBps / 100).toFixed(0)}%
        </p>
      </GlassCard>
    </div>
  );
}
