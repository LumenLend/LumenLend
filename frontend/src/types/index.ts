export interface UserPosition {
  assetSymbol: string;
  deposited: bigint;
  borrowed: bigint;
  depositValueUsd: bigint;
  borrowValueUsd: bigint;
}

export interface HealthFactorResult {
  value: bigint;
  totalCollateralUsd: bigint;
  totalDebtUsd: bigint;
}

export interface MarketState {
  symbol: string;
  totalSupply: bigint;
  totalBorrow: bigint;
  supplyApy: number;
  borrowApr: number;
  utilization: number;
}
