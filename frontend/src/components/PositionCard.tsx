"use client";

import { formatUsd, formatUnits } from "@/lib/format";

export function PositionCard({
  assetSymbol,
  deposited,
  borrowed,
  depositValueUsd,
  borrowValueUsd,
  decimals,
}: {
  assetSymbol: string;
  deposited: bigint;
  borrowed: bigint;
  depositValueUsd: bigint;
  borrowValueUsd: bigint;
  decimals: number;
}) {
  return (
    <div className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-base font-semibold">{assetSymbol}</h3>
        <span
          className="rounded-full bg-primary-600/20 px-2 py-0.5 text-xs font-medium text-primary-500"
          aria-label="Asset token"
        >
          {assetSymbol}
        </span>
      </div>

      <dl className="space-y-2 text-sm">
        <div className="flex items-center justify-between">
          <dt className="text-gray-400">Deposited</dt>
          <dd className="font-medium">
            {formatUnits(deposited, decimals)}
          </dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-gray-400">Deposit Value</dt>
          <dd className="font-medium text-green-400">
            {formatUsd(depositValueUsd)}
          </dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-gray-400">Borrowed</dt>
          <dd className="font-medium">{formatUnits(borrowed, decimals)}</dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="text-gray-400">Borrow Value</dt>
          <dd className="font-medium text-amber-400">
            {formatUsd(borrowValueUsd)}
          </dd>
        </div>
      </dl>
    </div>
  );
}