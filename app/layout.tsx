import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import { Exo_2, Geist_Mono, Inter, Orbitron, Rajdhani } from "next/font/google";
import { Toaster } from "@/components/ui/sonner";
import { cn } from "@/lib/utils";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

const orbitron = Orbitron({
  variable: "--font-orbitron",
  subsets: ["latin"],
});

const exo = Exo_2({
  variable: "--font-exo",
  subsets: ["latin"],
});

const rajdhani = Rajdhani({
  variable: "--font-rajdhani",
  subsets: ["latin"],
  weight: ["400", "600", "700"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Alien Protocol",
  description: "RWA lending on Stellar + Soroban.",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "Alien Protocol",
  },
  formatDetection: {
    telephone: false,
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
  themeColor: "#000000",
  colorScheme: "dark",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body
        className={cn(
          inter.variable,
          orbitron.variable,
          exo.variable,
          rajdhani.variable,
          geistMono.variable,
          "antialiased",
        )}
        suppressHydrationWarning
      >
        {children}
        <Toaster
          theme="dark"
          position="bottom-center"
          mobileOffset={{ bottom: "max(1rem, env(safe-area-inset-bottom))" }}
          offset={{ bottom: "1.5rem", right: "1rem" }}
        />
      </body>
    </html>
  );
}
