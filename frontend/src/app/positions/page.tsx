"use client";

import Link from "next/link";
import { SUPPORTED_ASSETS } from "@/lib/constants";
import { formatUsd } from "@/lib/format";
import { useWallet } from "@/hooks/useWallet";
import { usePositions } from "@/hooks/usePositions";
import { useHealthFactor } from "@/hooks/useLendingPool";
import { PositionCard } from "@/components/PositionCard";
import { HealthFactorBar } from "@/components/HealthFactorBar";

export default function PositionsPage() {
  const { address, isConnected } = useWallet();
  const { data: positions, isLoading, isError } = usePositions(address);
  const { data: healthFactor, isLoading: hfLoading } =
    useHealthFactor(address);

  if (!isConnected) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-bold">Your Positions</h2>
        <div className="rounded-lg border border-surface-light p-6 text-center text-sm text-gray-400">
          Connect your wallet to view your positions.
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Your Positions</h2>

      <section className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
        {hfLoading ? (
          <p className="text-sm text-gray-400">Loading health factor…</p>
        ) : (
          <HealthFactorBar
            healthFactor={healthFactor?.healthFactor ?? 0n}
          />
        )}
      </section>

      {isLoading ? (
        <p className="text-sm text-gray-400" role="status">
          Loading positions…
        </p>
      ) : isError ? (
        <div
          className="rounded-lg border border-red-800 bg-red-950/40 p-6 text-sm text-red-300"
          role="alert"
        >
          Failed to load your positions.
        </div>
      ) : !positions || positions.length === 0 ? (
        <div className="rounded-lg border border-surface-light p-6 text-center text-sm text-gray-400">
          You have no open positions yet.{" "}
          <Link
            href="/supply"
            className="text-primary-500 underline"
          >
            Supply an asset
          </Link>{" "}
          to get started.
        </div>
      ) : (
        <section className="grid gap-4 sm:grid-cols-2">
          {positions.map((position) => {
            const asset = SUPPORTED_ASSETS.find(
              (a) => a.symbol === position.assetSymbol
            );
            return (
              <PositionCard
                key={position.assetSymbol}
                assetSymbol={position.assetSymbol}
                deposited={position.deposited}
                borrowed={position.borrowed}
                depositValueUsd={position.depositValueUsd}
                borrowValueUsd={position.borrowValueUsd}
                decimals={asset?.decimals ?? 7}
              />
            );
          })}
        </section>
      )}

      <section className="rounded-lg border border-surface-light bg-surface-light/40 p-4">
        <div className="flex items-center justify-between text-sm font-medium text-gray-300">
          <span>Total Collateral (USD)</span>
          <span>{formatUsd(healthFactor?.totalCollateralUsd ?? 0n)}</span>
        </div>
        <div className="mt-2 flex items-center justify-between text-sm font-medium text-gray-300">
          <span>Total Debt (USD)</span>
          <span>{formatUsd(healthFactor?.totalDebtUsd ?? 0n)}</span>
        </div>
      </section>
    </div>
  );
}