"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { MenuIcon } from "lucide-react";
import { ConnectWalletButton } from "@/components/app/ConnectWalletButton";
import { PauseBanner } from "@/components/app/PauseBanner";
import { SpaceFx } from "@/components/app/Starfield";
import { TxModal } from "@/components/app/TxModal";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { cn } from "@/lib/utils";
import { useState, type ReactNode } from "react";

const NAV = [
  { href: "/app", label: "Dashboard" },
  { href: "/app/markets", label: "Markets" },
  { href: "/app/vault", label: "Vault" },
  { href: "/app/lend", label: "Lend" },
  { href: "/app/borrow", label: "Borrow" },
];

function isActive(path: string, href: string) {
  return href === "/app" ? path === "/app" : path.startsWith(href);
}

export function AppShell({ children }: { children: ReactNode }) {
  const path = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="app-root relative min-h-dvh text-white">
      <SpaceFx />
      <div className="relative z-10 flex min-h-dvh flex-col">
        <header className="fixed inset-x-0 top-0 z-50 border-b border-white/10 bg-black/90 pt-[env(safe-area-inset-top)] backdrop-blur-md">
          <div className="mx-auto flex h-16 max-w-[1440px] items-center gap-2 px-3 sm:h-20 sm:gap-4 sm:px-6 lg:gap-6 lg:px-8">
            <Link href="/" className="flex min-w-0 shrink-0 items-center gap-2 sm:gap-3">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src="/Alien-Protocol_2.png"
                alt=""
                width={36}
                height={36}
                className="h-8 w-8 mix-blend-screen sm:h-10 sm:w-10"
                style={{ animation: "glow-pulse 3s ease-in-out infinite" }}
              />
              <span className="hidden truncate font-orbitron text-[16px] font-extrabold uppercase tracking-[0.16em] shimmer-text sm:inline sm:text-[18px] sm:tracking-[0.2em]">
                Alien Protocol
              </span>
            </Link>

            <nav className="hidden min-w-0 flex-1 items-center justify-center gap-5 xl:gap-8 lg:flex">
              {NAV.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  data-active={isActive(path, item.href)}
                  className="nav-link"
                >
                  {item.label}
                </Link>
              ))}
            </nav>

            <div className="ml-auto flex shrink-0 items-center gap-1.5 sm:gap-2">
              <ConnectWalletButton compact />
              <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
                <SheetTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    className="size-10 lg:hidden"
                    aria-label={menuOpen ? "Close menu" : "Open menu"}
                  >
                    <MenuIcon className="size-4" />
                  </Button>
                </SheetTrigger>
                <SheetContent
                  side="right"
                  className="w-[min(100%,20rem)] border-white/15 bg-black p-0"
                >
                  <SheetHeader className="border-b border-white/10 px-4 py-5 text-left">
                    <SheetTitle className="font-orbitron text-sm uppercase tracking-[0.18em]">
                      Navigate
                    </SheetTitle>
                  </SheetHeader>
                  <nav className="flex flex-col px-2 py-2 pb-[max(1rem,env(safe-area-inset-bottom))]">
                    {NAV.map((item) => (
                      <Link
                        key={item.href}
                        href={item.href}
                        data-active={isActive(path, item.href)}
                        onClick={() => setMenuOpen(false)}
                        className={cn("nav-link !block px-3 py-3.5 text-base")}
                      >
                        {item.label}
                      </Link>
                    ))}
                  </nav>
                </SheetContent>
              </Sheet>
            </div>
          </div>
        </header>

        <div className="mx-auto flex w-full max-w-[1440px] flex-1 flex-col gap-4 px-3 pb-[max(2rem,env(safe-area-inset-bottom))] pt-[calc(4.5rem+env(safe-area-inset-top))] sm:px-6 sm:pb-10 sm:pt-[calc(6.25rem+env(safe-area-inset-top))] lg:px-8">
          <PauseBanner />
          <main className="app-fade min-w-0 flex-1">{children}</main>
        </div>
      </div>
      <TxModal />
    </div>
  );
}
