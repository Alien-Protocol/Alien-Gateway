import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

export function EmptyState({
  title,
  body,
  action,
}: {
  title: string;
  body: string;
  action?: ReactNode;
}) {
  return (
    <Card className="border-dashed border-white/25 bg-white/[0.02] py-0">
      <CardContent className="grid place-items-center px-6 py-12 text-center">
        <p className="font-orbitron text-lg tracking-wide text-white">{title}</p>
        <p className="mt-2 max-w-md text-base text-white/50">{body}</p>
        {action ? <div className="mt-5">{action}</div> : null}
      </CardContent>
    </Card>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return (
    <div className={cn("animate-pulse rounded-none bg-white/8", className ?? "h-8")} />
  );
}
