export interface SupportedAsset {
  symbol: string;
  name: string;
  decimals: number;
  ltv: number;
  liquidationThreshold: number;
  address: string;
}

export const SUPPORTED_ASSETS: SupportedAsset[] = [
  {
    symbol: "XLM",
    name: "Stellar Lumens",
    decimals: 7,
    ltv: 75,
    liquidationThreshold: 80,
    address: process.env.NEXT_PUBLIC_XLM_ASSET_ID ?? "",
  },
  {
    symbol: "USDC",
    name: "USD Coin",
    decimals: 6,
    ltv: 80,
    liquidationThreshold: 85,
    address: process.env.NEXT_PUBLIC_USDC_ASSET_ID ?? "",
  },
];

export const CONTRACT_IDS = {
  lendingPool: process.env.NEXT_PUBLIC_LENDING_POOL_CONTRACT_ID ?? "",
  interestRateModel:
    process.env.NEXT_PUBLIC_INTEREST_RATE_MODEL_CONTRACT_ID ?? "",
  liquidationEngine:
    process.env.NEXT_PUBLIC_LIQUIDATION_ENGINE_CONTRACT_ID ?? "",
  priceOracle: process.env.NEXT_PUBLIC_PRICE_ORACLE_CONTRACT_ID ?? "",
} as const;

export const STELLAR_NETWORK =
  process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? "testnet";

export const SOROBAN_RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ??
  "https://soroban-testnet.stellar.org";

export const HEALTH_FACTOR_SAFE = 1.5;
export const HEALTH_FACTOR_WARNING = 1.2;
export const HEALTH_FACTOR_DANGER = 1.0;

export const REFRESH_INTERVAL_MS = 30_000;

export const ASSET_BY_SYMBOL: Record<string, SupportedAsset> =
  SUPPORTED_ASSETS.reduce(
    (acc, asset) => {
      acc[asset.symbol] = asset;
      return acc;
    },
    {} as Record<string, SupportedAsset>
  );
