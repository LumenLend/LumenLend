import type { Metadata } from "next";
import type { ReactNode } from "react";
import { QueryClientProvider } from "@/lib/providers";
import "./globals.css";

export const metadata: Metadata = {
  title: "LumenLend — Decentralized Lending on Stellar",
  description:
    "A decentralized, non-custodial lending and borrowing protocol built on Stellar's Soroban smart contract platform.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-surface antialiased">
        <QueryClientProvider>
          <header className="sticky top-0 z-50 border-b border-surface-light bg-surface/80 backdrop-blur">
            <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
              <h1 className="text-lg font-bold">LumenLend</h1>
            </div>
          </header>
          <main className="mx-auto max-w-6xl px-4 py-8">{children}</main>
        </QueryClientProvider>
      </body>
    </html>
  );
}