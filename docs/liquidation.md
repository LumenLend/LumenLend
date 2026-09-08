# Liquidation Mechanism

When a borrower's position falls below the protocol's solvency threshold, it
becomes open to liquidation by anyone. This keeps the protocol solvent and
incentivizes a market of liquidators — no bot infrastructure is required at
the protocol level.

## Health Factor

The health factor expresses how much buffer a position has before it can be
liquidated.

```text
health_factor = (Σ collateral_i × price_i × liquidation_threshold_i / 10000) × 1e18
              / (Σ borrow_i × price_i)
```

- `health_factor >= 1e18` (1.0) → healthy.
- `health_factor < 1e18` → **liquidatable**.
- `i128::MAX` when the user has no debt.

> Collateral is discounted by each asset's `liquidation_threshold` (e.g. 80%),
> not its full value — this is the buffer that absorbs price volatility before
> a position becomes underwater.

## The Liquidation Engine

`contracts/liquidation_engine` implements:

| Function | Behavior |
|---|---|
| `is_liquidatable(borrower)` | Returns `get_health_factor(borrower) < 1e18` |
| `max_liquidatable_amount(borrower, debt_asset)` | Applies the 50% close factor to the outstanding borrow |
| `liquidate(liquidator, borrower, debt_asset, collateral_asset, debt_amount)` | Executes one liquidation |

### Close factor

At most **50%** of a borrower's outstanding debt in a single asset can be
repaid per transaction, preventing "liquidation griefing":

```rust
pub fn apply_close_factor(total_debt: i128) -> i128 {
    total_debt / 2
}
```

### Collateral seized

```text
collateral_to_seize = debt_amount * debt_price * (10000 + liquidation_bonus)
                    / (collateral_price * 10000)
```

The `liquidation_bonus` is read from the **collateral** asset's risk
configuration (the asset the liquidator receives). All prices are 18-decimal
fixed point; the USD precision cancels, leaving a collateral token amount in
fixed point. Intermediate products use checked `mul_div` to avoid overflow.

## Liquidation Flow

1. Liquidator calls `LiquidationEngine.liquidate(...)`; `liquidator.require_auth()`.
2. The engine verifies `get_health_factor(borrower) < 1e18` — panics otherwise.
3. It reads `get_user_borrow(borrower, debt_asset)` and caps `debt_amount` at
   the 50% close factor.
4. It fetches `debt_price` and `collateral_price` from the pool's oracle, and
   the collateral asset's liquidation bonus.
5. It computes `collateral_to_seize`.
6. It calls `LendingPool.execute_liquidation(...)` (auth-gated to the engine):
   the liquidator's funds repay the borrower's debt, and the collateral
   (plus bonus) is transferred to the liquidator.
7. It emits `(liquidation, liquidator, borrower, actual_debt)`.

The engine reads the pool through its own cross-contract helpers
(`get_health_factor`, `get_user_borrow`, `get_asset_price`,
`get_asset_liquidation_bonus`, `execute_liquidation`).

## Worked Example

Testnet risk parameters: XLM `liquidation_threshold` 80%, `liquidation_bonus`
5%. Prices: XLM `$0.10`, USDC `$1.00`.

**Healthy position**

- Collateral: 2,000 XLM = $200
- Borrow: 100 USDC = $100
- `health = 200 × 0.80 × 1e18 / 100 = 1.6e18` → healthy.

**Price crash** — XLM drops to `$0.06`.

- Collateral value: $120
- `health = 120 × 0.80 / 100 = 0.96e18` → **liquidatable**.

**Liquidator closes 50%** of the debt (50 USDC).

- Close factor: `100 / 2 = 50 USDC`.
- Seized collateral:
  `50 × $1.00 × 1.05 / $0.06 = 875 XLM`.
- Liquidator pays 50 USDC, receives 875 XLM worth $52.50 (a $2.50 bonus).

**After liquidation**

- Remaining collateral: 1,125 XLM = $67.50
- Remaining debt: 50 USDC
- `health = 67.5 × 0.80 / 50 = 1.08e18` → healthy again.

## Security Considerations

- Liquidations are fully oracle-dependent. The `PriceOracle` enforces a
  staleness bound (`max_price_age`, default 300s) so stale feeds cause a
  panic rather than a wrong liquidation.
- The bonus is deliberately small (5%, per-asset) to avoid profitable
  self-liquidation abuse while still attracting liquidators.
- The 50% close factor prevents one actor from wiping out an entire position
  and gaming the market in a single transaction.

## Recovered Funds

The protocol relies on liquidations to clear undercollateralized debt. If
liquidation is too slow and a price keeps falling, some bad debt may accrue.
Mitigations: conservative entry LTVs, steep `slope2` (see
[interest_rate_model.md](interest_rate_model.md)), and real-time monitoring.