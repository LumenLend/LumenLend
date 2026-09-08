#!/bin/bash
#
# generate_bindings.sh — Regenerates TypeScript Soroban contract bindings for
# the frontend from the deployed contracts, using the `stellar` CLI.
#
# Reads the deployed contract IDs from .env and outputs client classes into
# frontend/src/lib/contracts/. The directory is git-ignored and regenerated on
# demand (see README "Deploying to Testnet").
#
# Usage:
#   ./scripts/generate_bindings.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

# Load deployed contract IDs from .env.
if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
else
  echo "Error: .env not found. Deploy the contracts first (scripts/deploy_testnet.sh)." >&2
  exit 1
fi

: "${STELLAR_NETWORK:=testnet}"

OUT_DIR="frontend/src/lib/contracts"
mkdir -p "${OUT_DIR}"

NETWORK="${STELLAR_NETWORK}"

# Contract id -> output file name
declare -A CONTRACTS=(
  ["${LENDING_POOL_CONTRACT_ID:-}"]="lending_pool"
  ["${LTOKEN_XLM_CONTRACT_ID:-}"]="ltoken_xlm"
  ["${LTOKEN_USDC_CONTRACT_ID:-}"]="ltoken_usdc"
  ["${INTEREST_RATE_MODEL_CONTRACT_ID:-}"]="interest_rate_model"
  ["${LIQUIDATION_ENGINE_CONTRACT_ID:-}"]="liquidation_engine"
  ["${PRICE_ORACLE_CONTRACT_ID:-}"]="price_oracle"
)

generated=0
for contract_id in "${!CONTRACTS[@]}"; do
  name="${CONTRACTS[$contract_id]}"
  if [[ -z "$contract_id" ]]; then
    echo ">>> Skipping ${name}: no contract id in .env"
    continue
  fi

  echo ">>> Generating TypeScript bindings for ${name} (${contract_id})..."
  stellar contract bindings typescript \
    --id "${contract_id}" \
    --network "${NETWORK}" \
    --output-dir "${OUT_DIR}/${name}"
  generated=$((generated + 1))
done

echo ""
echo "Generated ${generated} binding set(s) into ${OUT_DIR}."
echo "Import them in the frontend, e.g.:"
echo "  import { Client } from '@/lib/contracts/lending_pool'"