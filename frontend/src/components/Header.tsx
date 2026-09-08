"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { WalletConnect } from "@/components/WalletConnect";

const NAV_LINKS = [
  { href: "/", label: "Markets" },
  { href: "/supply", label: "Supply" },
  { href: "/borrow", label: "Borrow" },
  { href: "/positions", label: "Positions" },
];

function linkClass(pathname: string, href: string): string {
  const active = pathname === href;
  return active
    ? "rounded-md bg-primary-600/20 px-3 py-1.5 text-sm font-medium text-primary-500"
    : "rounded-md px-3 py-1.5 text-sm font-medium text-gray-300 transition-colors hover:bg-surface-light hover:text-white";
}

export function Header() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-50 border-b border-surface-light bg-surface/80 backdrop-blur">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 px-4">
        <Link href="/" className="text-lg font-bold" aria-label="LumenLend home">
          LumenLend
        </Link>

        <nav aria-label="Primary" className="flex items-center gap-1">
          {NAV_LINKS.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={linkClass(pathname, link.href)}
            >
              {link.label}
            </Link>
          ))}
        </nav>

        <WalletConnect />
      </div>
    </header>
  );
}