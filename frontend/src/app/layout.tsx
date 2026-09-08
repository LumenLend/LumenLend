import type { Metadata } from "next";
import type { ReactNode } from "react";
import { QueryClientProvider } from "@/lib/providers";
import { Header } from "@/components/Header";
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
          <Header />
          <main className="mx-auto max-w-6xl px-4 py-8">{children}</main>
        </QueryClientProvider>
      </body>
    </html>
  );
}