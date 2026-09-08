"use client";

import { SUPPORTED_ASSETS } from "@/lib/constants";
import { formatApy, formatUnits } from "@/lib/format";
import { MarketRow } from "@/components/MarketRow";

export interface MarketColumn {
  symbol: string;
  totalSupply: string;
  supplyApy: string;
  totalBorrow: string;
  borrowApr: string;
  utilization: string;
}

export function MarketTable({
  rows,
  loading = false,
  error,
}: {
  rows?: MarketColumn[];
  loading?: boolean;
  error?: string | null;
}) {
  if (loading) {
    return (
      <div
        className="rounded-lg border border-surface-light p-6 text-center text-sm text-gray-400"
        role="status"
        aria-live="polite"
      >
        Loading markets…
      </div>
    );
  }

  if (error) {
    return (
      <div
        className="rounded-lg border border-red-800 bg-red-950/40 p-6 text-center text-sm text-red-300"
        role="alert"
      >
        Failed to load markets: {error}
      </div>
    );
  }

  const markets = rows ?? SUPPORTED_ASSETS.map((asset) => ({
    symbol: asset.symbol,
    totalSupply: formatUnits(0n, asset.decimals),
    supplyApy: formatApy(0n),
    totalBorrow: formatUnits(0n, asset.decimals),
    borrowApr: formatApy(0n),
    utilization: "0.00%",
  }));

  return (
    <div className="overflow-x-auto rounded-lg border border-surface-light">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-surface-light bg-surface-light/40 text-xs uppercase tracking-wider text-gray-400">
            <th className="px-4 py-3 font-medium">Asset</th>
            <th className="px-4 py-3 font-medium">Total Supply</th>
            <th className="px-4 py-3 font-medium">Supply APY</th>
            <th className="px-4 py-3 font-medium">Total Borrow</th>
            <th className="px-4 py-3 font-medium">Borrow APR</th>
            <th className="px-4 py-3 font-medium">Utilization</th>
          </tr>
        </thead>
        <tbody>
          {markets.map((row) => (
            <MarketRow key={row.symbol} row={row} />
          ))}
        </tbody>
      </table>
    </div>
  );
}