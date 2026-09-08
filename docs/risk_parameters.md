# Risk Parameters

Each supported asset carries its own risk configuration, stored per-asset in
`LendingPool` and set by the admin via `add_asset`. The configuration
determines how much borrowing power the asset provides (as collateral) and how
much is required (as a borrowed asset).

## Parameter Definitions

All ratios are stored as **basis points** (`1/10000`).

| Parameter | Meaning | Example |
|---|---|---|
| `ltv` (Loan-to-Value) | Maximum ratio of debt value to collateral value at which a user may borrow | `7500` = 75% |
| `liquidation_threshold` | Ratio at which a position becomes liquidatable | `8000` = 80% |
| `liquidation_bonus` | Bonus a liquidator receives for seizing the asset as collateral | `500` = 5% |
| `reserve_factor` | Protocol cut of interest paid by borrowers | `1000` = 10% |

- `ltv` < `liquidation_threshold` is required — the gap is the price-movement
  buffer between "you can't borrow more" and "you get liquidated".
- `liquidation_bonus` applies when the asset is used as **collateral**.
- `reserve_factor` scales the supply rate (see
  [interest_rate_model.md](interest_rate_model.md#supply-rate)).

## Default Testnet Parameters

Set by `scripts/deploy_testnet.sh`:

| Asset | LTV | Liquidation threshold | Liquidation bonus | Reserve factor |
|---|---|---|---|---|
| XLM (Stellar Lumens) | 75% (`7500`) | 80% (`8000`) | 5% (`500`) | 10% (`1000`) |
| USDC (USD Coin) | 80% (`8000`) | 85% (`8500`) | 5% (`500`) | 10% (`1000`) |

> Stables (USDC) can tolerate a higher LTV/threshold because their price is
> pegged; volatile assets (XLM) get more conservative values.

## How Parameters Govern Behavior

### Borrow capacity

The maximum a user can borrow against a collateral position:

```text
max_borrow = Σ (deposit_i × price_i × ltv_i / 10000)
```

### Liquidation boundary

```text
health_factor = Σ (deposit_i × price_i × liquidation_threshold_i / 10000) / Σ (borrow_i × price_i)

health_factor < 1.0  →  liquidatable
```

### Effective supply APY

```text
supply_apy = borrow_apy × utilization × (1 - reserve_factor)
```

## Admin Privileges

`add_asset` (and any future parameter update) requires `admin.require_auth()`.
In v1 the admin is a single address; the README security model calls for a
**multisig** and eventual DAO migration. Anyone can propose risk settings via
a GitHub discussion — see [CONTRIBUTING.md](../CONTRIBUTING.md).

## Risk Caps & Validation

- `add_asset` is admin-only and rejects assets that are already registered.
- The pool *stores* the parameters as-is; sensible bounds (`ltv <= 10_000`,
  `liquidation_threshold <= 10_000`, `ltv < liquidation_threshold`) are
  enforced by **governance/review**, not by the contract — a pre-mainnet
  hardening item tracked in the issue tracker.
- New assets should be activated only after an oracle feed exists and a review
  of the proposed parameters.

## Adding a New Asset

1. Deploy an lToken for the underlying asset.
2. Call `LendingPool.add_asset(asset, ltoken, ltv, liquidation_threshold,
   liquidation_bonus, reserve_factor)` as admin.
3. Add the asset to the Reflector oracle (price feed must track it).
4. Update `frontend/src/lib/constants.ts` with symbol/decimals/address and the
   `.env` asset IDs.
5. Add integration coverage: deposit → borrow → liquidate for the new asset.
6. Document the proposal in the repo and, before mainnet, in a formal
   community review.