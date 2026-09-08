export function formatUnits(
  amount: bigint,
  decimals: number,
  maxFractionDigits = 4
): string {
  const negative = amount < 0n;
  const abs = negative ? -amount : amount;
  const base = 10n ** BigInt(decimals);
  const whole = abs / base;
  const fraction = abs % base;

  const fractionStr = fraction.toString().padStart(decimals, "0").slice(0, maxFractionDigits);

  return `${negative ? "-" : ""}${whole}${
    fractionStr ? `.${fractionStr}` : ""
  }`;
}

export function parseUnits(value: string, decimals: number): bigint {
  const [whole = "", fraction = ""] = value.split(".");
  const base = 10n ** BigInt(decimals);
  const wholeNum = whole ? BigInt(whole) : 0n;
  const fractionPadded = fraction.padEnd(decimals, "0").slice(0, decimals);
  const fractionNum = fractionPadded ? BigInt(fractionPadded) : 0n;

  const negative = value.trim().startsWith("-");
  const result = wholeNum * base + (negative ? -fractionNum : fractionNum);
  return negative ? -result : result;
}

export function formatApy(rate18Decimals: string | number | bigint): string {
  const rate = BigInt(rate18Decimals);
  const basisPoints = Number(rate / 10n ** 14n) / 100;
  return `${basisPoints.toFixed(2)}%`;
}

export function formatUsd(amount: string | number | bigint): string {
  const value = BigInt(amount);
  const whole = value / 10n ** 18n;
  const fraction = (value % 10n ** 18n).toString().padStart(18, "0").slice(0, 2);
  return `$${whole.toString()}.${fraction}`;
}

export function truncateAddress(address: string, start = 6, end = 4): string {
  if (!address) return "";
  if (address.length <= start + end) return address;
  return `${address.slice(0, start)}…${address.slice(-end)}`;
}

export function formatHealthFactor(healthFactor: bigint): number {
  return Number(healthFactor) / 1e18;
}
