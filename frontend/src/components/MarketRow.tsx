"use client";

import type { MarketColumn } from "@/components/MarketTable";

export function MarketRow({ row }: { row: MarketColumn }) {
  return (
    <tr className="border-b border-surface-light last:border-0 hover:bg-surface-light/20">
      <td className="px-4 py-3 font-medium">{row.symbol}</td>
      <td className="px-4 py-3 text-gray-300">{row.totalSupply}</td>
      <td className="px-4 py-3 text-green-400">{row.supplyApy}</td>
      <td className="px-4 py-3 text-gray-300">{row.totalBorrow}</td>
      <td className="px-4 py-3 text-amber-400">{row.borrowApr}</td>
      <td className="px-4 py-3 text-gray-300">{row.utilization}</td>
    </tr>
  );
}