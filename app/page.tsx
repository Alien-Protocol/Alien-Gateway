"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import {
  FaGithub,
  FaLinkedinIn,
  FaTelegram,
  FaXTwitter,
} from "react-icons/fa6";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SpaceFx } from "@/components/app/Starfield";
import { cn } from "@/lib/utils";

const SOCIALS = [
  {
    href: "https://x.com/Alien_Proto",
    label: "X",
    icon: FaXTwitter,
  },
  {
    href: "https://t.me/alien_protocol_xyz",
    label: "Telegram",
    icon: FaTelegram,
  },
  {
    href: "https://www.linkedin.com/company/alien-protocol",
    label: "LinkedIn",
    icon: FaLinkedinIn,
  },
  {
    href: "https://github.com/Alien-Protocol",
    label: "GitHub",
    icon: FaGithub,
  },
] as const;

const FOOTER_LINKS = {
  links: [
    { href: "/app", label: "Launch App", external: false },
    { href: "/app/markets", label: "Markets", external: false },
    { href: "/app/vault", label: "Vault", external: false },
    { href: "/app/borrow", label: "Borrow", external: false },
  ],
  resources: [
    {
      href: "https://www.alien-protocol.xyz/",
      label: "Website",
      external: true,
    },
    {
      href: "https://github.com/Alien-Protocol",
      label: "GitHub",
      external: true,
    },
    { href: "/app/lend", label: "Lend", external: false },
    {
      href: "mailto:hello@alien-protocol.xyz",
      label: "Contact",
      external: true,
    },
  ],
  socials: [
    { href: "https://x.com/Alien_Proto", label: "X" },
    { href: "https://t.me/alien_protocol_xyz", label: "Telegram" },
    {
      href: "https://www.linkedin.com/company/alien-protocol",
      label: "LinkedIn",
    },
    { href: "https://github.com/Alien-Protocol", label: "GitHub" },
  ],
} as const;

function useTypewriter(texts: string[], speed = 60, pause = 2200) {
  const [display, setDisplay] = useState("");
  const [textIdx, setTextIdx] = useState(0);
  const [charIdx, setCharIdx] = useState(0);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => {
    const current = texts[textIdx];
    let timeout: ReturnType<typeof setTimeout>;

    if (!deleting && charIdx <= current.length) {
      timeout = setTimeout(() => {
        setDisplay(current.slice(0, charIdx));
        setCharIdx((c) => c + 1);
      }, speed);
    } else if (!deleting && charIdx > current.length) {
      timeout = setTimeout(() => setDeleting(true), pause);
    } else if (deleting && charIdx >= 0) {
      timeout = setTimeout(() => {
        setDisplay(current.slice(0, charIdx));
        setCharIdx((c) => c - 1);
      }, speed / 2);
    } else {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- existing typewriter cycle
      setDeleting(false);
      setTextIdx((i) => (i + 1) % texts.length);
    }
    return () => clearTimeout(timeout);
  }, [charIdx, deleting, textIdx, texts, speed, pause]);

  return display;
}

export default function Home() {
  const [email, setEmail] = useState("");
  const [submitted, setSubmitted] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [visible, setVisible] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email) return;
    setLoading(true);
    setError("");
    try {
      const res = await fetch("/api/notify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      if (!res.ok) throw new Error("Failed");
      setSubmitted(true);
    } catch {
      setError("Something went wrong. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  const typed = useTypewriter([
    "Turn tokenized RWAs into borrowing power",
    "Deposit treasuries. Borrow USDC. Stay in control",
    "Real-world collateral. On-chain credit. Built on Stellar",
    "One vault. One health factor. Capital that actually works",
  ]);

  useEffect(() => {
    const t = setTimeout(() => setVisible(true), 100);
    return () => clearTimeout(t);
  }, []);

  return (
    <div className="font-orbitron relative flex min-h-dvh flex-col overflow-x-clip bg-black text-white">
      <SpaceFx />

      <nav className="relative z-10 flex items-center justify-between gap-3 border-b border-white/10 px-4 py-4 pt-[max(1rem,env(safe-area-inset-top))] sm:px-9 sm:py-[22px] lg:px-[60px] lg:py-7">
        <div className="flex min-w-0 items-center gap-2.5 sm:gap-3">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src="/Alien-Protocol_2.png"
            alt="Alien Protocol Logo"
            width={40}
            height={40}
            className="block size-8 shrink-0 mix-blend-screen sm:size-10"
          />
          <span className="shimmer-text truncate font-orbitron text-[12px] font-extrabold tracking-[0.14em] uppercase sm:text-[15px] sm:tracking-[0.22em] lg:text-[17px]">
            Alien Protocol
          </span>
        </div>
        <Button asChild variant="outline" size="sm" className="h-10 shrink-0 px-3 sm:px-4">
          <Link href="/app">
            <span className="sm:hidden">App</span>
            <span className="hidden sm:inline">Launch App</span>
          </Link>
        </Button>
      </nav>

      <main className="relative z-10 flex flex-1 flex-col items-center justify-center px-4 py-12 text-center sm:px-5 sm:py-16">
        <div
          className={`fade-up mb-8 ${visible ? "visible" : ""}`}
          style={{
            animation: visible
              ? "fadeSlideUp 0.6s ease forwards, glow-pulse 3s ease-in-out 0.6s infinite"
              : undefined,
          }}
        >
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src="/Alien-Protocol_2.png"
            alt="Alien Protocol"
            className="mx-auto block size-[110px] mix-blend-screen sm:size-[140px] lg:size-[170px]"
          />
        </div>

        <p
          className={`fade-up font-raj mb-6 text-[11px] font-bold tracking-[0.45em] text-white/55 uppercase sm:text-[13px] lg:text-[15px] lg:tracking-[0.55em] ${visible ? "visible" : ""}`}
          style={{ animationDelay: "0.15s" }}
        >
          Alien Protocol
        </p>

        <div
          className={`fade-up network-pill mb-9 gap-3.5 px-5 py-2.5 text-[15px] tracking-[0.28em] sm:px-[30px] sm:py-3.5 sm:text-xl sm:tracking-[0.4em] lg:px-11 lg:py-4 lg:text-[26px] lg:tracking-[0.48em] ${visible ? "visible" : ""}`}
          style={{
            animationDelay: "0.28s",
            animation: visible
              ? "fadeSlideUp 0.7s 0.28s ease forwards, badge-flicker 8s 1s infinite"
              : undefined,
          }}
        >
          <span className="dot-blink inline-block" />
          <span className="font-orbitron font-black text-white uppercase">
            Coming Soon
          </span>
          <span
            className="dot-blink inline-block"
            style={{ animationDelay: "0.7s" }}
          />
        </div>

        <div
          className={`fade-up font-exo mb-11 flex min-h-[52px] max-w-[310px] items-center justify-center text-[13px] leading-[1.85] font-medium tracking-[0.04em] text-white/78 sm:min-h-[58px] sm:max-w-[430px] sm:text-base lg:min-h-16 lg:max-w-[540px] lg:text-lg ${visible ? "visible" : ""}`}
          style={{ animationDelay: "0.42s" }}
        >
          <span>{typed}</span>
          <span className="cursor-blink" />
        </div>

        {!submitted ? (
          <div
            className={`fade-up w-full max-w-[320px] sm:max-w-[420px] lg:max-w-[520px] ${visible ? "visible" : ""}`}
            style={{ animationDelay: "0.55s" }}
          >
            <form
              onSubmit={handleSubmit}
              className="flex w-full flex-col border border-white/32 sm:h-14 sm:flex-row sm:items-stretch"
            >
              <Input
                type="email"
                autoComplete="email"
                enterKeyHint="send"
                placeholder="your@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="h-12 min-w-0 flex-1 rounded-none border-0 border-b border-white/20 bg-transparent px-3.5 text-base tracking-[0.06em] shadow-none focus-visible:ring-0 sm:h-full sm:border-b-0 sm:px-5 sm:text-sm lg:px-[22px] lg:text-[15px]"
              />
              <Button
                type="submit"
                className="h-12 w-full shrink-0 animate-none rounded-none px-4 text-xs sm:h-full sm:w-auto sm:px-6 lg:px-7 lg:text-[13px]"
              >
                {loading ? "SENDING..." : "Notify Me"}
              </Button>
            </form>
            {error ? (
              <p className="font-exo mt-2.5 text-[13px] tracking-[0.06em] text-red-400/85">
                ⚠ {error}
              </p>
            ) : null}
          </div>
        ) : (
          <div className="animate-[fadeSlideUp_0.5s_ease_forwards] border border-white/32 px-9 py-[18px] font-orbitron text-xs font-bold tracking-[0.28em] text-white/75 uppercase sm:text-[15px]">
            ✓ &nbsp; You&apos;re on the list
          </div>
        )}

        <p
          className={`fade-up font-raj mt-3.5 text-[11px] font-semibold tracking-[0.22em] text-white/32 uppercase sm:text-sm ${visible ? "visible" : ""}`}
          style={{ animationDelay: "0.65s" }}
        >
          No spam. Launch announcement only.
        </p>
      </main>

      <footer className="relative z-10 border-t border-white/10 pb-[env(safe-area-inset-bottom)]">
        <div className="mx-auto max-w-[1440px] px-4 py-10 sm:px-9 sm:py-16 lg:px-12 lg:py-20">
          <div className="grid grid-cols-2 gap-8 sm:gap-10 md:grid-cols-[minmax(0,1.35fr)_repeat(3,minmax(0,0.7fr))] md:gap-10 lg:gap-16">
            <div className="col-span-2 max-w-md md:col-span-1">
              <div className="flex items-center gap-3">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img
                  src="/Alien-Protocol_2.png"
                  alt=""
                  className="size-8 mix-blend-screen sm:size-9"
                />
                <span className="font-orbitron text-lg font-bold tracking-wide text-white sm:text-xl">
                  Alien Protocol
                </span>
              </div>
              <p className="font-exo mt-5 text-sm leading-relaxed text-white/80 sm:text-[15px]">
                Discover, access, and activate tokenized real-world assets —
                deposit collateral, borrow USDC, and earn on Stellar.
              </p>
              <a
                href="mailto:hello@alien-protocol.xyz"
                className="font-exo mt-5 inline-block text-sm text-white/45 transition-colors hover:text-white/80"
              >
                hello@alien-protocol.xyz
              </a>
            </div>

            <FooterCol title="Links" items={FOOTER_LINKS.links} />
            <FooterCol title="Resources" items={FOOTER_LINKS.resources} />
            <FooterCol
              title="Socials"
              items={FOOTER_LINKS.socials.map((s) => ({
                ...s,
                external: true,
              }))}
            />
          </div>

          <div className="mt-14 flex flex-col-reverse items-start justify-between gap-6 border-t border-white/10 pt-7 sm:mt-16 sm:flex-row sm:items-center">
            <p className="font-exo text-sm text-white/40">
              2026 Alien Protocol — All Rights Reserved
            </p>
            <div className="flex items-center gap-3">
              {SOCIALS.map(({ href, label, icon: Icon }) => (
                <a
                  key={href}
                  href={href}
                  target="_blank"
                  rel="noreferrer"
                  aria-label={label}
                  title={label}
                  className={cn(
                    "inline-flex size-10 items-center justify-center rounded-full",
                    "bg-white/10 text-white/85 transition-colors",
                    "hover:bg-white/20 hover:text-white",
                  )}
                >
                  <Icon className="size-4" aria-hidden />
                </a>
              ))}
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

function FooterCol({
  title,
  items,
}: {
  title: string;
  items: readonly {
    href: string;
    label: string;
    external?: boolean;
  }[];
}) {
  return (
    <div>
      <h3 className="font-orbitron text-[11px] font-bold tracking-[0.22em] text-white uppercase">
        {title}
      </h3>
      <ul className="mt-5 space-y-3">
        {items.map((item) => (
          <li key={`${title}-${item.label}`}>
            {item.external ? (
              <a
                href={item.href}
                target={item.href.startsWith("mailto:") ? undefined : "_blank"}
                rel={
                  item.href.startsWith("mailto:") ? undefined : "noreferrer"
                }
                className="font-exo text-sm text-white/45 transition-colors hover:text-white"
              >
                {item.label}
              </a>
            ) : (
              <Link
                href={item.href}
                className="font-exo text-sm text-white/45 transition-colors hover:text-white"
              >
                {item.label}
              </Link>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
