"use client";

import { useQuery } from "@tanstack/react-query";
import { CONTRACT_IDS, SUPPORTED_ASSETS } from "@/lib/constants";
import { readContract } from "@/lib/contract-call";
import type { UserPosition } from "@/types";

function toBigInt(value: unknown): bigint {
  return typeof value === "bigint"
    ? value
    : typeof value === "number"
      ? BigInt(value)
      : typeof value === "string"
        ? BigInt(value)
        : 0n;
}

export function usePositions(address: string) {
  return useQuery<UserPosition[]>({
    queryKey: ["positions", address],
    queryFn: async (): Promise<UserPosition[]> => {
      if (!address || !CONTRACT_IDS.lendingPool) return [];

      const positions: UserPosition[] = [];

      for (const asset of SUPPORTED_ASSETS) {
        const assetId = asset.address;

        const [depositsResult, borrowsResult] = await Promise.all([
          readContract({
            contractId: CONTRACT_IDS.lendingPool,
            method: "user_deposits",
            args: [address, assetId],
          }),
          readContract({
            contractId: CONTRACT_IDS.lendingPool,
            method: "user_borrows",
            args: [address, assetId],
          }),
        ]);

        const deposited = depositsResult.isOk
          ? toBigInt(depositsResult.value)
          : 0n;
        const borrowed = borrowsResult.isOk
          ? toBigInt(borrowsResult.value)
          : 0n;

        if (deposited === 0n && borrowed === 0n) continue;

        positions.push({
          assetSymbol: asset.symbol,
          deposited,
          borrowed,
          depositValueUsd: deposited,
          borrowValueUsd: borrowed,
        });
      }

      return positions;
    },
    enabled: Boolean(address) && Boolean(CONTRACT_IDS.lendingPool),
  });
}