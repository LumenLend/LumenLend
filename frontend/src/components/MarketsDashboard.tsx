"use client";

import { SUPPORTED_ASSETS } from "@/lib/constants";
import { formatApy, formatUnits } from "@/lib/format";
import { MarketTable, type MarketColumn } from "@/components/MarketTable";
import { useWallet } from "@/hooks/useWallet";
import { useTotalCollateral, useTotalDebt } from "@/hooks/useLendingPool";
import { usePrices } from "@/hooks/usePrices";
import { usePositions } from "@/hooks/usePositions";
import { formatUsd } from "@/lib/format";

function marketColumnFor(
  asset: (typeof SUPPORTED_ASSETS)[number],
  positions: ReturnType<typeof usePositions>["data"]
): MarketColumn {
  const position = positions?.find((p) => p.assetSymbol === asset.symbol);
  const totalSupply = position?.deposited ?? 0n;
  const totalBorrow = position?.borrowed ?? 0n;
  const utilization =
    totalSupply > 0n ? (Number(totalBorrow) / Number(totalSupply)) * 100 : 0;

  return {
    symbol: asset.symbol,
    totalSupply: formatUnits(totalSupply, asset.decimals),
    supplyApy: formatApy(0n),
    totalBorrow: formatUnits(totalBorrow, asset.decimals),
    borrowApr: formatApy(0n),
    utilization: `${utilization.toFixed(2)}%`,
  };
}

export function MarketsDashboard() {
  const { address } = useWallet();
  const { data: collateral } = useTotalCollateral(address);
  const { data: debt } = useTotalDebt(address);
  const { data: prices } = usePrices();
  const { data: positions } = usePositions(address);

  const markets = SUPPORTED_ASSETS.map((asset) =>
    marketColumnFor(asset, positions)
  );

  const vaultTvl = collateral ?? 0n;

  return (
    <div className="space-y-6">
      <section className="grid gap-4 sm:grid-cols-3">
        <div className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
          <p className="text-xs uppercase tracking-wider text-gray-400">
            Protocol TVL
          </p>
          <p className="mt-1 text-xl font-semibold">{formatUsd(vaultTvl)}</p>
        </div>
        <div className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
          <p className="text-xs uppercase tracking-wider text-gray-400">
            Total Collateral
          </p>
          <p className="mt-1 text-xl font-semibold">
            {formatUsd(collateral ?? 0n)}
          </p>
        </div>
        <div className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
          <p className="text-xs uppercase tracking-wider text-gray-400">
            Total Debt
          </p>
          <p className="mt-1 text-xl font-semibold text-amber-400">
            {formatUsd(debt ?? 0n)}
          </p>
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-2xl font-bold">Markets</h2>
        <MarketTable rows={markets} loading={!prices} />
      </section>
    </div>
  );
}