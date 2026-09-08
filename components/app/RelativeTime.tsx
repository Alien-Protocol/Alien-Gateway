"use client";

import { relativeTime } from "@/lib/format";
import { useEffect, useState } from "react";

/** Renders after mount so `Date.now()` cannot mismatch SSR HTML. */
export function RelativeTime({
  at,
  fallback = "—",
}: {
  at: number;
  fallback?: string;
}) {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    const tick = () => setLabel(relativeTime(at));
    tick();
    const id = setInterval(tick, 15_000);
    return () => clearInterval(id);
  }, [at]);

  return <>{label ?? fallback}</>;
}
