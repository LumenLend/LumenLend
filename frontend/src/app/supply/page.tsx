"use client";

import { useMemo, useState } from "react";
import { SUPPORTED_ASSETS } from "@/lib/constants";
import { formatUnits, parseUnits } from "@/lib/format";
import { useWallet } from "@/hooks/useWallet";
import { useDeposit, useHealthFactor } from "@/hooks/useLendingPool";
import { HealthFactorBar } from "@/components/HealthFactorBar";
import { TransactionModal } from "@/components/TransactionModal";

export default function SupplyPage() {
  const { address, isConnected } = useWallet();
  const [assetSymbol, setAssetSymbol] = useState(SUPPORTED_ASSETS[0].symbol);
  const [amount, setAmount] = useState("");
  const [confirming, setConfirming] = useState(false);
  const deposit = useDeposit();

  const asset = useMemo(
    () => SUPPORTED_ASSETS.find((a) => a.symbol === assetSymbol) ?? SUPPORTED_ASSETS[0],
    [assetSymbol]
  );

  const { data: healthFactor } = useHealthFactor(address);

  const depositAmount = useMemo(() => {
    if (!amount) return 0n;
    const parsed = parseUnits(amount, asset.decimals);
    return parsed > 0n ? parsed : 0n;
  }, [amount, asset.decimals]);

  const apyPreview = "Est. APY: —";

  function handleConfirm() {
    if (!address || !asset.address) return;
    setConfirming(true);
    deposit
      .mutateAsync({
        from: address,
        asset: asset.address,
        amount: amount,
      })
      .catch(() => {
        // surface error via UI state
      })
      .finally(() => setConfirming(false));
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Supply Assets</h2>

      {!isConnected ? (
        <div className="rounded-lg border border-surface-light p-6 text-center text-sm text-gray-400">
          Connect your wallet to supply assets.
        </div>
      ) : (
        <div className="max-w-lg space-y-4 rounded-lg border border-surface-light bg-surface-light/40 p-6">
          <label className="block">
            <span className="text-sm font-medium text-gray-300">Asset</span>
            <select
              value={assetSymbol}
              onChange={(e) => setAssetSymbol(e.target.value)}
              className="mt-1 w-full rounded-md border border-surface-light bg-surface px-3 py-2 text-sm"
              aria-label="Select asset to supply"
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
              aria-label={`Amount of ${asset.symbol} to supply`}
            />
            <span className="mt-1 block text-xs text-gray-500">
              Balance: {formatUnits(0n, asset.decimals)} {asset.symbol}
            </span>
          </label>

          <p className="text-sm text-gray-400">{apyPreview}</p>

          {(healthFactor?.healthFactor ?? 0n) > 0n && (
            <HealthFactorBar healthFactor={healthFactor?.healthFactor ?? 1_000_000_000_000_000_000n} />
          )}

          <button
            type="button"
            onClick={() => depositAmount > 0n && setConfirming(true)}
            disabled={depositAmount <= 0n || !asset.address}
            className="w-full rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-60"
          >
            Supply {asset.symbol}
          </button>
        </div>
      )}

      <TransactionModal
        open={confirming}
        title="Confirm Supply"
        assetSymbol={asset.symbol}
        amount={amount || "0"}
        estimatedGas="~0.02 XLM"
        onConfirm={handleConfirm}
        onCancel={() => setConfirming(false)}
        pending={deposit.isPending}
      />
    </div>
  );
}