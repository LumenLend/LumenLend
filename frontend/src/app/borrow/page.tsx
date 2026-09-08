"use client";

import { useMemo, useState } from "react";
import { SUPPORTED_ASSETS, HEALTH_FACTOR_DANGER } from "@/lib/constants";
import { parseUnits } from "@/lib/format";
import { useWallet } from "@/hooks/useWallet";
import { useBorrow, useHealthFactor } from "@/hooks/useLendingPool";
import { HealthFactorBar } from "@/components/HealthFactorBar";
import { TransactionModal } from "@/components/TransactionModal";

export default function BorrowPage() {
  const { address, isConnected } = useWallet();
  const [assetSymbol, setAssetSymbol] = useState(SUPPORTED_ASSETS[0].symbol);
  const [amount, setAmount] = useState("");
  const [confirming, setConfirming] = useState(false);
  const borrow = useBorrow();

  const asset = useMemo(
    () =>
      SUPPORTED_ASSETS.find((a) => a.symbol === assetSymbol) ??
      SUPPORTED_ASSETS[0],
    [assetSymbol]
  );

  const { data: healthFactor } = useHealthFactor(address);
  const currentHf: bigint =
    healthFactor && healthFactor.healthFactor > 0n
      ? healthFactor.healthFactor
      : 1_000_000_000_000_000_000n;

  const borrowUnits = useMemo(() => {
    if (!amount) return 0n;
    const parsed = parseUnits(amount, asset.decimals);
    return parsed > 0n ? parsed : 0n;
  }, [amount, asset.decimals]);

  const postBorrowHf = useMemo(() => {
    if (!healthFactor || healthFactor.healthFactor <= 0n) return currentHf;
    const current = Number(healthFactor.healthFactor) / 1e18;
    const borrowValue = Number(borrowUnits) / 10 ** asset.decimals;
    const estimated = current * (1 / (1 + borrowValue));
    const clamped = Math.max(Number(HEALTH_FACTOR_DANGER), estimated);
    return BigInt(Math.floor(clamped * 1e18));
  }, [healthFactor, borrowUnits, asset.decimals, currentHf]);

  const wouldLiquidate =
    (healthFactor?.healthFactor ?? 0n) > 0n &&
    borrowUnits > 0n &&
    postBorrowHf < BigInt(HEALTH_FACTOR_DANGER * 1e18);

  function handleConfirm() {
    if (!address || !asset.address) return;
    setConfirming(true);
    borrow
      .mutateAsync({ from: address, asset: asset.address, amount })
      .catch(() => {})
      .finally(() => setConfirming(false));
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Borrow Assets</h2>

      {!isConnected ? (
        <div className="rounded-lg border border-surface-light p-6 text-center text-sm text-gray-400">
          Connect your wallet to borrow assets.
        </div>
      ) : (
        <div className="max-w-lg space-y-4 rounded-lg border border-surface-light bg-surface-light/40 p-6">
          <label className="block">
            <span className="text-sm font-medium text-gray-300">Asset</span>
            <select
              value={assetSymbol}
              onChange={(e) => setAssetSymbol(e.target.value)}
              className="mt-1 w-full rounded-md border border-surface-light bg-surface px-3 py-2 text-sm"
              aria-label="Select asset to borrow"
            >
              {SUPPORTED_ASSETS.map((supported) => (
                <option value={supported.symbol} key={supported.symbol}>
                  {supported.symbol} — {supported.name}
                </option>
              ))}
            </select>
          </label>

          <label className="block">
            <span className="text-sm font-medium text-gray-300">
              Amount ({asset.symbol})
            </span>
            <input
              type="number"
              min="0"
              step="any"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.0"
              className="mt-1 w-full rounded-md border border-surface-light bg-surface px-3 py-2 text-sm"
              aria-label={`Amount of ${asset.symbol} to borrow`}
            />
          </label>

          <div className="space-y-3">
            <div>
              <h3 className="mb-1 text-sm font-medium text-gray-300">
                Current Health Factor
              </h3>
              <HealthFactorBar healthFactor={currentHf} />
            </div>
            <div>
              <h3 className="mb-1 text-sm font-medium text-gray-300">
                Post-Borrow Health Factor
              </h3>
              <HealthFactorBar healthFactor={postBorrowHf} />
            </div>
          </div>

          {wouldLiquidate && (
            <p className="text-sm text-red-400" role="alert">
              This borrow would put your position below the liquidation
              threshold.
            </p>
          )}

          <button
            type="button"
            onClick={() => borrowUnits > 0n && !wouldLiquidate && setConfirming(true)}
            disabled={borrowUnits <= 0n || wouldLiquidate || !asset.address}
            className="w-full rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-60"
          >
            Borrow {asset.symbol}
          </button>
        </div>
      )}

      <TransactionModal
        open={confirming}
        title="Confirm Borrow"
        assetSymbol={asset.symbol}
        amount={amount || "0"}
        estimatedGas="~0.02 XLM"
        healthFactorAfter={`${(Number(postBorrowHf) / 1e18).toFixed(2)}`}
        onConfirm={handleConfirm}
        onCancel={() => setConfirming(false)}
        pending={borrow.isPending}
      />
    </div>
  );
}