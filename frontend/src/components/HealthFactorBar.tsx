"use client";

import {
  HEALTH_FACTOR_DANGER,
  HEALTH_FACTOR_SAFE,
  HEALTH_FACTOR_WARNING,
} from "@/lib/constants";
import { formatHealthFactor } from "@/lib/format";

function healthFactorColor(hf: number): string {
  if (hf >= HEALTH_FACTOR_SAFE) return "bg-green-500";
  if (hf >= HEALTH_FACTOR_WARNING) return "bg-amber-400";
  return "bg-red-500";
}

export function HealthFactorBar({
  healthFactor,
}: {
  healthFactor: bigint;
}) {
  const hf = formatHealthFactor(healthFactor);
  const pct = Math.min(Math.max(hf / 2.5, 0), 1) * 100;
  const color = healthFactorColor(hf);

  return (
    <div
      className="space-y-1"
      role="meter"
      aria-valuemin={0}
      aria-valuemax={2.5}
      aria-valuenow={hf}
      aria-label={`Health factor ${hf.toFixed(2)}`}
    >
      <div className="flex items-center justify-between text-sm">
        <span className="text-gray-400">Health Factor</span>
        <span className="font-semibold tabular-nums">{hf.toFixed(2)}</span>
      </div>
      <div className="h-2 w-full overflow-hidden rounded-full bg-surface-light">
        <div
          className={`h-full rounded-full transition-all ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <div className="flex justify-between text-[10px] uppercase tracking-wider text-gray-500">
        <span>Liquidated below {HEALTH_FACTOR_DANGER.toFixed(1)}</span>
        <span>Safe above {HEALTH_FACTOR_SAFE.toFixed(1)}</span>
      </div>
    </div>
  );
}