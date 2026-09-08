"use client";

import { useWallet } from "@/hooks/useWallet";
import { truncateAddress } from "@/lib/format";

export function WalletConnect() {
  const { address, isConnected, isConnecting, connect, disconnect } =
    useWallet();

  if (isConnected && address) {
    return (
      <div className="flex items-center gap-3">
        <span
          className="rounded-md border border-surface-light bg-surface-light/60 px-3 py-1.5 text-sm font-medium text-gray-200"
          aria-label={`Connected wallet ${address}`}
          title={address}
        >
          {truncateAddress(address)}
        </span>
        <button
          type="button"
          onClick={disconnect}
          className="rounded-md border border-gray-600 px-3 py-1.5 text-sm font-medium text-gray-300 transition-colors hover:border-gray-400 hover:text-white"
        >
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={connect}
      disabled={isConnecting}
      className="rounded-md bg-primary-600 px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-60"
      aria-label="Connect Stellar wallet"
    >
      {isConnecting ? "Connecting…" : "Connect Wallet"}
    </button>
  );
}