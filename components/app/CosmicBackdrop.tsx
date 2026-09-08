"use client";

/** Shared deep-space scenery — black & white only. */
export function CosmicBackdrop() {
  return (
    <div
      className="pointer-events-none fixed inset-0 z-[1] overflow-hidden grayscale"
      aria-hidden
    >
      {/* Galaxies */}
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src="/galaxy-nebula.png"
        alt=""
        className="absolute -left-[8%] top-[4%] h-[40vmin] w-auto max-w-none animate-spin-slow opacity-30 mix-blend-screen sm:opacity-40"
      />
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src="/galaxy-nebula.png"
        alt=""
        className="absolute -right-[14%] bottom-[6%] h-[46vmin] w-auto max-w-none animate-spin-slower opacity-25 mix-blend-screen sm:opacity-35"
      />

      {/* Voyager satellite pass */}
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src="/voyager-satellite.png"
        alt=""
        className="absolute top-[30%] h-12 w-auto max-w-none opacity-60 mix-blend-screen animate-voyager-pass sm:h-20 sm:opacity-75 lg:h-24"
      />
    </div>
  );
}
