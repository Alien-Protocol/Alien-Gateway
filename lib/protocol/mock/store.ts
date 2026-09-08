"use client";

import { ADDRESSES } from "@/lib/protocol/constants";
import {
  holdingsToCollateral,
  collateralValueUsd,
  utilizationBps,
} from "@/lib/protocol/math";
import { createSeed } from "@/lib/protocol/mock/seed";
import { derivePosition as positionFromSnapshot, listUsers } from "@/lib/protocol/selectors";
import type { Position, ProtocolSnapshot } from "@/lib/protocol/types";

export type ProtocolState = ProtocolSnapshot;

let state: ProtocolState = createSeed();
const listeners = new Set<() => void>();

function emit() {
  listeners.forEach((l) => l());
}

export function getState(): ProtocolState {
  return state;
}

export function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function setState(
  patch: Partial<ProtocolState> | ((s: ProtocolState) => ProtocolState),
) {
  state = typeof patch === "function" ? patch(state) : { ...state, ...patch };
  emit();
}

export function resetState() {
  state = createSeed();
  emit();
}

/**
 * Clone the demo "you" bag onto a real wallet address so mock balances
 * remain usable after connecting Freighter / Lobstr / etc.
 */
export function adoptConnectedUser(address: string) {
  if (!address || address === ADDRESSES.you) return;
  setState((s) => {
    const cloneKey = <T>(bag: Record<string, T>): Record<string, T> => {
      if (bag[address] != null) return bag;
      const seed = bag[ADDRESSES.you];
      if (seed == null) return bag;
      return { ...bag, [address]: structuredClone(seed) };
    };
    return {
      ...s,
      wallets: cloneKey(s.wallets),
      holdings: cloneKey(s.holdings),
      debts: cloneKey(s.debts),
      supplies: cloneKey(s.supplies),
    };
  });
}

export function derivePosition(user: string, s: ProtocolState = state): Position {
  return positionFromSnapshot(user, s);
}

export function allUsers(): string[] {
  return listUsers(state);
}

export function refreshPoolDerived(s: ProtocolState): ProtocolState {
  const collateralTvl = allUsersFrom(s).reduce((sum, u) => {
    const c = holdingsToCollateral(s.holdings[u] ?? {}, s.prices, s.assets);
    return sum + collateralValueUsd(c);
  }, 0);
  const available = Math.max(0, s.pool.totalSupply - s.pool.totalBorrowed);
  return {
    ...s,
    pool: {
      ...s.pool,
      availableLiquidity: available,
      utilizationBps: utilizationBps(s.pool.totalBorrowed, s.pool.totalSupply),
      tvlUsd: collateralTvl + s.pool.totalSupply,
    },
  };
}

function allUsersFrom(s: ProtocolState): string[] {
  return [
    ...new Set([...Object.keys(s.holdings), ...Object.keys(s.debts), ADDRESSES.you]),
  ];
}

export function pushEvent(
  event: Omit<ProtocolState["events"][number], "id" | "txHash" | "status"> & {
    id?: string;
    txHash?: string;
    status?: ProtocolState["events"][number]["status"];
  },
) {
  const id = event.id ?? `e${Date.now().toString(36)}`;
  setState((s) => ({
    ...s,
    events: [
      {
        status: "success",
        txHash: event.txHash ?? id.repeat(4).slice(0, 64),
        ...event,
        id,
      },
      ...s.events,
    ],
  }));
}
