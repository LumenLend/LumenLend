#!/bin/bash
#
# deploy_testnet.sh — Full testnet deployment sequence for LumenLend.
#
# Builds all Soroban contracts to WASM, deploys them to Stellar testnet via
# the `stellar` CLI, initializes every contract with the correct parameters,
# registers the supported assets, and writes the resulting contract IDs back
# into .env for the frontend to consume.
#
# Prerequisites:
#   - `stellar` CLI installed and a testnet deployer account funded
#     (soroban keys generate --global testnet-deployer --network testnet)
#   - `.env` present with STELLAR_NETWORK, DEPLOYER_SECRET_KEY, and
#     REFLECTOR_CONTRACT_ID set.
#
# Usage:
#   ./scripts/deploy_testnet.sh

set -euo pipefail

# ---------------------------------------------------------------- config ----

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

# Load environment (never commit real secret keys).
if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
else
  echo "Error: .env not found. Copy .env.example to .env and fill in values." >&2
  exit 1
fi

: "${STELLAR_NETWORK:=testnet}"
: "${DEPLOYER_SECRET_KEY:?DEPLOYER_SECRET_KEY must be set in .env}"
: "${REFLECTOR_CONTRACT_ID:?REFLECTOR_CONTRACT_ID must be set in .env}"

NETWORK="${STELLAR_NETWORK}"
WASM_DIR="target/wasm32-unknown-unknown/release"
# The admin should be a public G-address. If not configured, derive it from the
# deployer key so the `stellar` CLI can sign the privileged calls.
ADMIN_ADDRESS="${ADMIN_ADDRESS:-}"
if [[ -z "$ADMIN_ADDRESS" ]]; then
  if [[ "$DEPLOYER_SECRET_KEY" == G* ]]; then
    ADMIN_ADDRESS="$DEPLOYER_SECRET_KEY"
  else
    ADMIN_ADDRESS="$(stellar keys address "$DEPLOYER_SECRET_KEY" 2>/dev/null || echo "$DEPLOYER_SECRET_KEY")"
  fi
fi
ORACLE_ADDRESS="$PRICE_ORACLE_CONTRACT_ID"
IRM_ADDRESS="$INTEREST_RATE_MODEL_CONTRACT_ID"
LENDING_POOL_ADDRESS="$LENDING_POOL_CONTRACT_ID"
LIQUIDATION_ENGINE_ADDRESS="$LIQUIDATION_ENGINE_CONTRACT_ID"

declare -A WASM=( )
WASM[lending_pool]="lending_pool.wasm"
WASM[ltoken]="ltoken.wasm"
WASM[interest_rate_model]="interest_rate_model.wasm"
WASM[liquidation_engine]="liquidation_engine.wasm"
WASM[price_oracle]="price_oracle.wasm"

# --------------------------------------------------------------- helpers ----

deploy_contract() {
  local name="$1"
  local wasm="${WASM[$name]}"
  echo ">> Deploying ${name} (${wasm})..."
  local contract_id
  contract_id="$(stellar contract deploy \
    --wasm "${WASM_DIR}/${wasm}" \
    --source "${DEPLOYER_SECRET_KEY}" \
    --network "${NETWORK}")"
  echo "   ${name} -> ${contract_id}"
  echo "${contract_id}"
}

invoke() {
  local contract_id="$1"
  shift
  echo ">> invoke $*"
  stellar contract invoke \
    --id "${contract_id}" \
    --source "${DEPLOYER_SECRET_KEY}" \
    --network "${NETWORK}" \
    -- "$@"
}

write_env() {
  local key="$1"
  local value="$2"
  if [[ -f .env ]]; then
    if grep -q "^${key}=" .env; then
      sed -i "s|^${key}=.*|${key}=${value}|" .env
    else
      echo "${key}=${value}" >> .env
    fi
    echo "   .env updated: ${key}=${value}"
  fi
}

# ------------------------------------------------------------------ build ----

echo "=== Building contracts (wasm32 release) ==="
cargo build --target wasm32-unknown-unknown --release

mkdir -p "${WASM_DIR}"

# ---------------------------------------------------------------- deploy ----

echo "=== Deploying contracts ==="

if [[ -z "$ORACLE_ADDRESS" ]]; then
  PRICE_ORACLE_CONTRACT_ID="$(deploy_contract "price_oracle")"
  ORACLE_ADDRESS="${PRICE_ORACLE_CONTRACT_ID}"
  write_env "PRICE_ORACLE_CONTRACT_ID" "${PRICE_ORACLE_CONTRACT_ID}"
fi

if [[ -z "$IRM_ADDRESS" ]]; then
  INTEREST_RATE_MODEL_CONTRACT_ID="$(deploy_contract "interest_rate_model")"
  IRM_ADDRESS="${INTEREST_RATE_MODEL_CONTRACT_ID}"
  write_env "INTEREST_RATE_MODEL_CONTRACT_ID" "${INTEREST_RATE_MODEL_CONTRACT_ID}"
fi

if [[ -z "$LENDING_POOL_ADDRESS" ]]; then
  LENDING_POOL_CONTRACT_ID="$(deploy_contract "lending_pool")"
  LENDING_POOL_ADDRESS="${LENDING_POOL_CONTRACT_ID}"
  write_env "LENDING_POOL_CONTRACT_ID" "${LENDING_POOL_CONTRACT_ID}"
fi

if [[ -z "$LIQUIDATION_ENGINE_ADDRESS" ]]; then
  LIQUIDATION_ENGINE_CONTRACT_ID="$(deploy_contract "liquidation_engine")"
  LIQUIDATION_ENGINE_ADDRESS="${LIQUIDATION_ENGINE_CONTRACT_ID}"
  write_env "LIQUIDATION_ENGINE_CONTRACT_ID" "${LIQUIDATION_ENGINE_CONTRACT_ID}"
fi

if [[ -z "$LTOKEN_XLM_CONTRACT_ID" ]]; then
  LTOKEN_XLM_CONTRACT_ID="$(deploy_contract "ltoken")"
  write_env "LTOKEN_XLM_CONTRACT_ID" "${LTOKEN_XLM_CONTRACT_ID}"
fi

if [[ -z "$LTOKEN_USDC_CONTRACT_ID" ]]; then
  LTOKEN_USDC_CONTRACT_ID="$(deploy_contract "ltoken")"
  write_env "LTOKEN_USDC_CONTRACT_ID" "${LTOKEN_USDC_CONTRACT_ID}"
fi

# ------------------------------------------------------------ initialize ----

echo "=== Initializing contracts ==="

# InterestRateModel: base_rate=200 (2%), slope1=1000 (10%), slope2=30000 (300%),
# optimal_utilization=8000 (80%).
invoke "${INTEREST_RATE_MODEL_CONTRACT_ID}" \
  initialize \
  --base_rate 200 \
  --slope1 1000 \
  --slope2 30000 \
  --optimal_utilization 8000

# PriceOracle: wrap the Reflector contract.
invoke "${PRICE_ORACLE_CONTRACT_ID}" \
  initialize \
  --reflector_contract "${REFLECTOR_CONTRACT_ID}"

# lTokens (XLM + USDC). The underlying asset ids should be added to .env as
# XLM_ASSET_CONTRACT_ID and USDC_ASSET_CONTRACT_ID before running.
: "${XLM_ASSET_CONTRACT_ID:?XLM_ASSET_CONTRACT_ID must be set in .env}"
: "${USDC_ASSET_CONTRACT_ID:?USDC_ASSET_CONTRACT_ID must be set in .env}"

invoke "${LTOKEN_XLM_CONTRACT_ID}" \
  initialize \
  --lending_pool "${LENDING_POOL_CONTRACT_ID}" \
  --underlying_asset "${XLM_ASSET_CONTRACT_ID}" \
  --name "LumenLend Stellar Lumens" \
  --symbol "lXLM"

invoke "${LTOKEN_USDC_CONTRACT_ID}" \
  initialize \
  --lending_pool "${LENDING_POOL_CONTRACT_ID}" \
  --underlying_asset "${USDC_ASSET_CONTRACT_ID}" \
  --name "LumenLend USD Coin" \
  --symbol "lUSDC"

# LendingPool: point at the oracle and the interest rate model.
invoke "${LENDING_POOL_CONTRACT_ID}" \
  initialize \
  --admin "${ADMIN_ADDRESS}" \
  --oracle "${PRICE_ORACLE_CONTRACT_ID}" \
  --interest_rate_model "${INTEREST_RATE_MODEL_CONTRACT_ID}"

# LendingPool: register supported assets.
# XLM: ltv=7500 (75%), liquidation_threshold=8000 (80%), bonus=500 (5%), reserve=1000 (10%).
invoke "${LENDING_POOL_CONTRACT_ID}" \
  add_asset \
  --asset "${XLM_ASSET_CONTRACT_ID}" \
  --ltoken_address "${LTOKEN_XLM_CONTRACT_ID}" \
  --ltv 7500 \
  --liquidation_threshold 8000 \
  --liquidation_bonus 500 \
  --reserve_factor 1000

# USDC: ltv=8000 (80%), liquidation_threshold=8500 (85%), bonus=500 (5%), reserve=1000 (10%).
invoke "${LENDING_POOL_CONTRACT_ID}" \
  add_asset \
  --asset "${USDC_ASSET_CONTRACT_ID}" \
  --ltoken_address "${LTOKEN_USDC_CONTRACT_ID}" \
  --ltv 8000 \
  --liquidation_threshold 8500 \
  --liquidation_bonus 500 \
  --reserve_factor 1000

# LiquidationEngine: register the lending pool, then authorize the pool.
invoke "${LIQUIDATION_ENGINE_CONTRACT_ID}" \
  initialize \
  --lending_pool "${LENDING_POOL_CONTRACT_ID}"

invoke "${LENDING_POOL_CONTRACT_ID}" \
  set_liquidation_engine \
  --engine "${LIQUIDATION_ENGINE_CONTRACT_ID}"

# --------------------------------------------------------------- summary ----

echo ""
echo "=== Deployment summary (testnet) ==="
echo "LENDING_POOL_CONTRACT_ID=${LENDING_POOL_CONTRACT_ID}"
echo "INTEREST_RATE_MODEL_CONTRACT_ID=${INTEREST_RATE_MODEL_CONTRACT_ID}"
echo "PRICE_ORACLE_CONTRACT_ID=${PRICE_ORACLE_CONTRACT_ID}"
echo "LIQUIDATION_ENGINE_CONTRACT_ID=${LIQUIDATION_ENGINE_CONTRACT_ID}"
echo "LTOKEN_XLM_CONTRACT_ID=${LTOKEN_XLM_CONTRACT_ID}"
echo "LTOKEN_USDC_CONTRACT_ID=${LTOKEN_USDC_CONTRACT_ID}"
echo ""
echo "Contract IDs written to .env for the frontend (NEXT_PUBLIC_*). Done."