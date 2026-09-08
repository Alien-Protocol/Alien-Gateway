"use client";

import { cancelTxSign, confirmTxSign, useTx } from "@/context/TxContext";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { truncateAddress } from "@/lib/format";
import { cn } from "@/lib/utils";

export function TxModal() {
  const { open, phase, request, error, txHash, expertUrl, close } = useTx();
  if (!request) return null;

  const busy = phase === "signing" || phase === "pending";

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next && phase !== "signing" && phase !== "pending") close();
      }}
    >
      <DialogContent showCloseButton={false} className="gap-0 sm:max-w-md">
        <DialogHeader>
          <p className="font-orbitron text-[11px] font-semibold uppercase tracking-[0.22em] text-white/55">
            Confirm transaction
          </p>
          <DialogTitle className="mt-2 font-orbitron text-lg font-semibold tracking-wide text-white">
            {request.title}
          </DialogTitle>
          <DialogDescription className="mt-2 text-sm leading-relaxed text-white/60">
            {request.detail}
          </DialogDescription>
        </DialogHeader>

        <div className="mt-5 space-y-2 border border-white/15 bg-black p-3 text-sm">
          <Row
            k="Status"
            v={
              phase === "idle"
                ? "Awaiting signature"
                : phase === "signing"
                  ? "Signing with wallet…"
                  : phase === "pending"
                    ? "Broadcasting to mock network…"
                    : phase === "success"
                      ? "Confirmed"
                      : "Failed"
            }
          />
          {txHash ? <Row k="Tx" v={truncateAddress(txHash, 8, 8)} /> : null}
          {error ? <p className="text-sm text-red-300">{error}</p> : null}
        </div>

        <DialogFooter className="mt-5 mx-0 mb-0 rounded-none border-0 bg-transparent p-0 sm:justify-start">
          {phase === "idle" ? (
            <>
              <Button type="button" onClick={() => confirmTxSign()}>
                Sign with wallet
              </Button>
              <Button type="button" variant="outline" onClick={() => cancelTxSign()}>
                Cancel
              </Button>
            </>
          ) : null}
          {busy ? (
            <Button type="button" disabled>
              {phase === "signing" ? "Signing…" : "Pending…"}
            </Button>
          ) : null}
          {phase === "success" ? (
            <>
              {expertUrl ? (
                <Button type="button" variant="outline" asChild>
                  <a href={expertUrl} target="_blank" rel="noreferrer">
                    View on Stellar Expert ↗
                  </a>
                </Button>
              ) : null}
              <Button type="button" onClick={close}>
                Done
              </Button>
            </>
          ) : null}
          {phase === "error" ? (
            <Button type="button" onClick={close}>
              Close
            </Button>
          ) : null}
        </DialogFooter>
        <p className="mt-4 text-[11px] text-white/35">
          Confirm in your wallet to continue.
        </p>
      </DialogContent>
    </Dialog>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-white/40">{k}</span>
      <span className={cn("tabular-nums text-white/85")}>{v}</span>
    </div>
  );
}
