"use client";

import { useQuery } from "@tanstack/react-query";
import { CONTRACT_IDS, SUPPORTED_ASSETS } from "@/lib/constants";
import { readContract } from "@/lib/contract-call";

function toBigInt(value: unknown): bigint {
  return typeof value === "bigint"
    ? value
    : typeof value === "number"
      ? BigInt(value)
      : typeof value === "string"
        ? BigInt(value)
        : 0n;
}

export interface AssetPrice {
  symbol: string;
  priceUsd: bigint;
}

export function usePrices() {
  return useQuery<AssetPrice[]>({
    queryKey: ["prices"],
    queryFn: async (): Promise<AssetPrice[]> => {
      if (!CONTRACT_IDS.priceOracle) {
        // Fall back to nominal 1:1 prices so the UI remains usable pre-deploy.
        return SUPPORTED_ASSETS.map((asset) => ({
          symbol: asset.symbol,
          priceUsd: 10n ** 18n,
        }));
      }

      const prices: AssetPrice[] = [];

      for (const asset of SUPPORTED_ASSETS) {
        if (!asset.address) {
          prices.push({ symbol: asset.symbol, priceUsd: 10n ** 18n });
          continue;
        }
        const result = await readContract({
          contractId: CONTRACT_IDS.priceOracle,
          method: "get_price",
          args: [asset.address],
        });
        prices.push({
          symbol: asset.symbol,
          priceUsd: result.isOk ? toBigInt(result.value) : 10n ** 18n,
        });
      }

      return prices;
    },
    refetchInterval: 30_000,
  });
}