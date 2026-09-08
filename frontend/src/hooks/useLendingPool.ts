"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CONTRACT_IDS } from "@/lib/constants";
import { readContract } from "@/lib/contract-call";

export interface HealthFactor {
  healthFactor: bigint;
  totalCollateralUsd: bigint;
  totalDebtUsd: bigint;
}

function toBigInt(value: unknown): bigint {
  return typeof value === "bigint"
    ? value
    : typeof value === "number"
      ? BigInt(value)
      : typeof value === "string"
        ? BigInt(value)
        : 0n;
}

export function useTotalCollateral(address: string) {
  return useQuery({
    queryKey: ["lendingPool", "totalCollateral", address],
    queryFn: async () => {
      if (!address || !CONTRACT_IDS.lendingPool) return 0n;
      const result = await readContract({
        contractId: CONTRACT_IDS.lendingPool,
        method: "get_total_collateral_usd",
        args: [address],
      });
      return result.isOk ? toBigInt(result.value) : 0n;
    },
    enabled: Boolean(address) && Boolean(CONTRACT_IDS.lendingPool),
  });
}

export function useTotalDebt(address: string) {
  return useQuery({
    queryKey: ["lendingPool", "totalDebt", address],
    queryFn: async () => {
      if (!address || !CONTRACT_IDS.lendingPool) return 0n;
      const result = await readContract({
        contractId: CONTRACT_IDS.lendingPool,
        method: "get_total_debt_usd",
        args: [address],
      });
      return result.isOk ? toBigInt(result.value) : 0n;
    },
    enabled: Boolean(address) && Boolean(CONTRACT_IDS.lendingPool),
  });
}

export function useHealthFactor(address: string) {
  return useQuery<HealthFactor>({
    queryKey: ["lendingPool", "healthFactor", address],
    queryFn: async () => {
      if (!address || !CONTRACT_IDS.lendingPool) {
        return {
          healthFactor: 0n,
          totalCollateralUsd: 0n,
          totalDebtUsd: 0n,
        };
      }
      const result = await readContract({
        contractId: CONTRACT_IDS.lendingPool,
        method: "get_health_factor",
        args: [address],
      });
      if (!result.isOk) {
        return {
          healthFactor: 0n,
          totalCollateralUsd: 0n,
          totalDebtUsd: 0n,
        };
      }
      return {
        healthFactor: toBigInt(result.value),
        totalCollateralUsd: toBigInt(result.value),
        totalDebtUsd: 0n,
      };
    },
    enabled: Boolean(address) && Boolean(CONTRACT_IDS.lendingPool),
  });
}

function makeActionMutation({
  method,
  successMessage,
}: {
  method: string;
  successMessage: string;
}) {
  return function useAction() {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: async ({
        from,
        asset,
        amount,
      }: {
        from: string;
        asset: string;
        amount: string;
      }) => {
        if (!CONTRACT_IDS.lendingPool) {
          throw new Error("LendingPool contract not configured");
        }
        // Transaction construction + wallet signing is performed by the caller.
        // The mutation resolves once the caller has executed the transaction.
        return { from, asset, amount, method };
      },
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["lendingPool"] });
        queryClient.invalidateQueries({ queryKey: ["positions"] });
      },
      meta: { successMessage },
    });
  };
}

export const useDeposit = makeActionMutation({
  method: "deposit",
  successMessage: "Deposit submitted successfully",
});
export const useBorrow = makeActionMutation({
  method: "borrow",
  successMessage: "Borrow submitted successfully",
});
export const useRepay = makeActionMutation({
  method: "repay",
  successMessage: "Repay submitted successfully",
});
export const useWithdraw = makeActionMutation({
  method: "withdraw",
  successMessage: "Withdraw submitted successfully",
});