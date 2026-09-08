# Interest Rate Model

LumenLend uses a **kinked (two-slope) utilization-based interest rate model**,
conceptually identical to Compound v2 / Aave v2. It is designed to keep
utilization in a healthy range: rates rise gently toward the optimal
utilization ("the kink") and then steeply above it, discouraging
over-utilization of a market's liquidity.

## Configuration

The model is configured once during `initialize` and stored as **basis
points** (`1/10000`):

| Parameter | Meaning | Testnet value | Bound |
|---|---|---|---|
| `base_rate` | Minimum borrow rate at 0% utilization | `200` = 2% APR | ≤ 10 000 |
| `slope1` | Rate added at the kink (gradual below-kink slope) | `1000` = 10% | ≤ 1 000 000 |
| `slope2` | Rate added for excess utilization above the kink | `30000` = 300% | ≤ 1 000 000 |
| `optimal_utilization` | Target utilization ratio | `8000` = 80% | 0 < x ≤ 10 000 |

All returned rates are annualized, in **18-decimal fixed point** (`1e18` ==
100%).

## Utilization

```text
utilization = total_borrows * 1e18 / total_deposits
```

- Capped at `1e18` (100%).
- Returns `0` when `total_deposits <= 0` or `total_borrows <= 0`.

## Borrow Rate

```text
util  <= optimal:  borrow_rate = base_rate + slope1 *       (util / optimal)
util  >  optimal:  borrow_rate = base_rate + slope1 + slope2 * (util - optimal) / (1 - optimal)
```

At the testnet parameters:

| Utilization | Borrow rate |
|---|---|
| 0% | 2.00% |
| 40% | 7.00% |
| 80% (kink) | 12.00% |
| 90% | 162.00% |
| 100% | 312.00% |

> The below-kink term interpolates `slope1` linearly from `0` at 0% utilization
> to `slope1` at the kink (this matches Compound v2 behavior more closely than
> the "base + slope1 * util" form in early README diagrams).

## Supply Rate

```text
supply_rate = borrow_rate * utilization * (1 - reserve_factor)
```

The `reserve_factor` (basis points, e.g. `1000` = 10%) is the protocol's cut
of the interest paid by borrowers; the remainder accrues to depositors.

### Worked example (80% utilization, 10% reserve)

```text
borrow_rate = 12.00% = 0.12
utilization = 80%    = 0.80
reserve     = 10%    = 0.10

supply_rate = 0.12 * 0.80 * (1 - 0.10) = 0.0864 = 8.64% APR
```

## Fixed-Point Implementation Notes

- Precision: `SCALE = 1e18`; basis points are converted via
  `from_basis_points` (`bps * 1e18 / 10000`).
- All intermediate products use checked `mul_div` to prevent `i128` overflow.
- The model is pure and stateless after initialization — rates depend only on
  current `total_deposits` / `total_borrows`, making them trivial to simulate
  and test.

## Integration

The `LendingPool` calls `get_borrow_rate` / `get_supply_rate` during interest
accrual on every state-changing operation (`deposit`, `borrow`, `repay`,
`withdraw`). See [docs/architecture.md](architecture.md#3-interest--rate-flow).

## Changing Parameters

Parameters are immutable in v1. Adjusting the model requires either a
redeployment or a future upgrade path — a deliberate safety choice for a
non-upgradeable protocol.