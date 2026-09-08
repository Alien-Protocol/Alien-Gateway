"use client";

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { toast } from "sonner";
import { appConfig } from "@/lib/config";
import { fakeTxHash } from "@/lib/format";
import { ProtocolError } from "@/lib/protocol/errors";

export type TxPhase = "idle" | "signing" | "pending" | "success" | "error";

export type TxRequest = {
  title: string;
  detail: string;
};

export type Toast = {
  id: string;
  message: string;
  tone: "success" | "error" | "info";
};

type TxContextValue = {
  open: boolean;
  phase: TxPhase;
  request: TxRequest | null;
  error: string | null;
  txHash: string | null;
  expertUrl: string | null;
  toasts: Toast[];
  execute: (
    request: TxRequest,
    fn: () => Promise<{ txHash?: string } | void>,
    successToast?: string,
  ) => Promise<boolean>;
  close: () => void;
  dismissToast: (id: string) => void;
  pushToast: (message: string, tone?: Toast["tone"]) => void;
};

const TxContext = createContext<TxContextValue | null>(null);

export function TxProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const [phase, setPhase] = useState<TxPhase>("idle");
  const [request, setRequest] = useState<TxRequest | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const pushToast = useCallback(
    (message: string, tone: Toast["tone"] = "success") => {
      if (tone === "error") toast.error(message);
      else if (tone === "info") toast.info(message);
      else toast.success(message);
    },
    [],
  );

  const dismissToast = useCallback((id: string) => {
    void id;
    toast.dismiss();
  }, []);

  const close = useCallback(() => {
    setOpen(false);
    setPhase("idle");
    setError(null);
    setTxHash(null);
    setRequest(null);
  }, []);

  const execute = useCallback(
    async (
      req: TxRequest,
      fn: () => Promise<{ txHash?: string } | void>,
      successToast?: string,
    ) => {
      setRequest(req);
      setOpen(true);
      setPhase("idle");
      setError(null);
      setTxHash(null);

      return new Promise<boolean>((resolve) => {
        pendingResolver = async () => {
          setPhase("signing");
          await new Promise((r) => setTimeout(r, 650));
          setPhase("pending");
          try {
            const result = await fn();
            const hash = result?.txHash ?? fakeTxHash(req.title + Date.now());
            setTxHash(hash);
            setPhase("success");
            if (successToast) pushToast(successToast, "success");
            resolve(true);
          } catch (err) {
            const message =
              err instanceof ProtocolError
                ? err.message
                : err instanceof Error
                  ? err.message
                  : "Transaction failed";
            setError(message);
            setPhase("error");
            pushToast(message, "error");
            resolve(false);
          }
        };
        cancelResolver = () => {
          close();
          resolve(false);
        };
      });
    },
    [close, pushToast],
  );

  const expertUrl = txHash ? `${appConfig.explorerTxUrl}/${txHash}` : null;

  const value = useMemo(
    () => ({
      open,
      phase,
      request,
      error,
      txHash,
      expertUrl,
      toasts: [] as Toast[],
      execute,
      close,
      dismissToast,
      pushToast,
    }),
    [
      open,
      phase,
      request,
      error,
      txHash,
      expertUrl,
      execute,
      close,
      dismissToast,
      pushToast,
    ],
  );

  return <TxContext.Provider value={value}>{children}</TxContext.Provider>;
}

let pendingResolver: (() => void) | null = null;
let cancelResolver: (() => void) | null = null;

export function confirmTxSign() {
  pendingResolver?.();
}

export function cancelTxSign() {
  cancelResolver?.();
}

export function useTx() {
  const ctx = useContext(TxContext);
  if (!ctx) throw new Error("useTx must be used within TxProvider");
  return ctx;
}
