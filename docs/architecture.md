# LumenLend Architecture

> Expanding on the README [Architecture](https://github.com/YOUR_GITHUB_USERNAME/lumenlend-protocol#architecture)
> section: contract interaction diagram, data flow, storage schema, and
> cross-contract call patterns.

LumenLend is a non-custodial lending protocol built on Stellar's Soroban smart
contract platform. This document describes how the five contracts fit together,
how money and data move through the system, and the patterns used to keep the
protocol safe and composable.

---

## 1. Component Overview

| Contract | Role | Key Interfaces |
|---|---|---|
| `LendingPool` | Central entry point; holds funds, tracks positions | `deposit`, `borrow`, `repay`, `withdraw`, `get_health_factor` |
| `LToken` | SEP-41 receipt token minted to depositors | `mint`, `burn`, `balance`, `exchange_rate` |
| `InterestRateModel` | Computes utilization-based rates | `get_borrow_rate`, `get_supply_rate`, `get_utilization_rate` |
| `LiquidationEngine` | Liquidates undercollateralized positions | `liquidate`, `is_liquidatable`, `max_liquidatable_amount` |
| `PriceOracle` | Wraps Reflector for USD price feeds | `get_price`, `get_prices`, `last_updated` |

```
                         ┌──────────────────────────┐
                         │     User / Frontend       │
                         │  (Freighter Wallet)        │
                         └────────────┬─────────────┘
                                      │
                         ┌────────────▼─────────────┐
                         │      LendingPool          │
                         │  (Core Protocol Entry)    │
                         ├──────────────────────────┤
                         │  deposit() borrow()       │
                         │  repay()  withdraw()      │
                         │  get_health_factor()      │
                         └──┬──────────┬─────────────┘
                            │          │
              ┌─────────────▼──┐  ┌────▼──────────────┐
              │  LToken        │  │  InterestRateModel │
              │  (per asset)   │  │                    │
              │  mint() burn() │  │  get_borrow_rate() │
              │  balance_of()  │  │  get_supply_rate() │
              │  exchange_rate │  └────────────────────┘
              └────────────────┘
                            │
              ┌─────────────▼──────────┐   ┌────────────────────┐
              │  LiquidationEngine     │   │  PriceOracle       │
              │  liquidate()           │◄──│  get_price()       │
              │  is_liquidatable()     │   │  get_prices()      │
              └────────────────────────┘   └────────────────────┘
```

---

## 2. Storage Schema

All contracts use Soroban instance storage (`env.storage().instance()`) for
configuration and persistent storage for per-user data, keyed by enums that
derive `#[contracttype]`.

### LendingPool

**Instance storage (config):**

| Key | Type | Description |
|---|---|---|
| `Admin` | `Address` | Contract admin (multisig before DAO migration) |
| `Oracle` | `Address` | PriceOracle contract address |
| `InterestRateModel` | `Address` | IRM contract address |
| `LiquidationEngine` | `Address` | LiquidationEngine contract address |

**Persistent storage:**

| Key | Type | Description |
|---|---|---|
| `AssetConfig(Asset)` | `AssetConfig` | Risk params: `ltv`, `liquidation_threshold`, `liquidation_bonus`, `reserve_factor`, `ltoken_address`, `is_active` |
| `AssetState(Asset)` | `AssetState` | `total_deposits`, `total_borrows`, `last_update_timestamp`, `borrow_index`, `deposit_index` |
| `UserDeposit(User, Asset)` | `i128` | Deposited amount per user per asset |
| `UserBorrow(User, Asset)` | `i128` | Outstanding borrow per user per asset |

```rust
pub struct AssetConfig {
    pub ltv: u32,                  // 7500 = 75%
    pub liquidation_threshold: u32,// 8000 = 80%
    pub liquidation_bonus: u32,    // 500  = 5%
    pub reserve_factor: u32,       // 1000 = 10%
    pub ltoken_address: Address,
    pub is_active: bool,
}

pub struct AssetState {
    pub total_deposits: i128,
    pub total_borrows: i128,
    pub last_update_timestamp: u64,
    pub borrow_index: i128,        // 18-decimal index
    pub deposit_index: i128,
}
```

### LToken

| Key | Type | Description |
|---|---|---|
| `LendingPool` | `Address` | The only address allowed to mint/burn |
| `UnderlyingAsset` | `Address` | The asset this receipt token wraps |
| `Name` / `Symbol` | `String` | e.g. `lXLM` |
| `TotalSupply` | `i128` | Total receipt tokens outstanding |
| `ExchangeRate` | `i128` | `(total_underlying * 1e18) / total_supply` |
| `Balance(Address)` | `i128` | Per-owner lToken balance |

The exchange rate starts at `1e18` (1 lToken = 1 underlying) and grows as
interest accrues, so one lToken redeems for more underlying over time.

### InterestRateModel / PriceOracle / LiquidationEngine

| Contract | Key | Type |
|---|---|---|
| `InterestRateModel` | `BaseRate` / `Slope1` / `Slope2` / `OptimalUtilization` | `u32` (basis points / 10000) |
| `PriceOracle` | `ReflectorContract` / `MaxPriceAge` | `Address` / `u64` (seconds, default 300) |
| `LiquidationEngine` | `LendingPool` | `Address` |

---

## 3. Interest & Rate Flow

Every state-changing call (`deposit`, `borrow`, `repay`, `withdraw`) begins
by accruing interest:

```
accrue_interest(asset):
  elapsed        = now - state.last_update_timestamp
  borrow_rate    = irm.get_borrow_rate(total_deposits, total_borrows)
  borrow_index  *= 1 + borrow_rate * elapsed / SECONDS_PER_YEAR
  deposit_index *= 1 + supply_rate * elapsed / SECONDS_PER_YEAR
  total_borrows  = borrows_outstanding * borrow_index
  state.last_update_timestamp = now
```

Rates come from the kinked two-slope model:

```
utilization   = total_borrows * 1e18 / total_deposits        (0 at no deposits)
borrow_rate   = base_rate + slope1 * utilization                     (below kink)
              = base_rate + slope1 * optimal + slope2 * (util - optimal) (above kink)
supply_rate   = borrow_rate * utilization * (1 - reserve_factor)
```

All math uses 18-decimal `i128` fixed-point with checked operations.

---

## 4. Data Flow

### 4.1 Deposit

1. User calls `LendingPool.deposit(depositor, asset, amount)`.
2. `depositor.require_auth()` gates the call.
3. `accrue_interest(asset)` runs first.
4. Protocol transfers `amount` of `asset` from the depositor (SEP-41
   `transfer_from`-style flow via a token client on `asset`).
5. lTokens minted: `ltokens = amount * 1e18 / exchange_rate`.
6. `LToken.mint` (auth-gated to the pool) credits the depositor.
7. `AssetState.total_deposits += amount`; `UserDeposit += amount`.
8. Emits `(deposit, depositor, asset, amount)`.

### 4.2 Borrow

1. User calls `LendingPool.borrow(borrower, asset, amount)`.
2. `borrower.require_auth()`; `accrue_interest(asset)`.
3. Guards: asset active, `total_deposits - total_borrows >= amount`.
4. Health check: post-borrow health factor must stay `>= 1e18`.
5. Protocol transfers `amount` out to the borrower.
6. `AssetState.total_borrows += amount`; `UserBorrow += amount`.
7. Emits `(borrow, borrower, asset, amount)`.

### 4.3 Repay

1. `LendingPool.repay(repayer, asset, amount)` with `repayer.require_auth()`.
2. `accrue_interest(asset)`; amount is capped at the outstanding borrow.
3. Transfers `amount` to the protocol; reduces `total_borrows` and `UserBorrow`.
4. If borrow reaches `0`, the `UserBorrow` storage entry is cleaned up.
5. Emits `(repay, repayer, asset, amount)`.

### 4.4 Withdraw

1. `LendingPool.withdraw(withdrawer, asset, amount)` with `require_auth()`.
2. `accrue_interest(asset)`; checks lToken balance, protocol liquidity, and
   post-withdrawal health factor `>= 1e18`.
3. Burns lTokens (auth-gated), transfers underlying to the withdrawer.
4. Reduces `total_deposits` and `UserDeposit`.
5. Emits `(withdraw, withdrawer, asset, amount)`.

### 4.5 Liquidation

1. `LiquidationEngine.liquidate(liquidator, borrower, debt_asset, collateral_asset, debt_amount)`.
2. Cross-calls `LendingPool.get_health_factor(borrower)`; panics if `>= 1e18`.
3. Caps `debt_amount` at `max_liquidatable_amount` (close factor = 50% of debt).
4. Computes collateral to seize:
   `collateral = debt_amount * debt_price * (10000 + bonus) / (collateral_price * 10000)`.
5. Calls back into `LendingPool.execute_liquidation` (auth-gated to the engine),
   which repays the debt with the liquidator's funds and transfers collateral
   (+ bonus) to the liquidator.
6. Emits `(liquidation, liquidator, borrower, debt_amount)`.

### 4.6 Health Factor

```
health_factor = sum(deposit_i * price_i * threshold_i / 10000) * 1e18
              / sum(borrow_i * price_i)

health_factor < 1e18   →  liquidatable
```

- `get_total_collateral_usd(user)`: `sum(deposit_i * price_i)`.
- `get_total_debt_usd(user)`: `sum(borrow_i * price_i)`.
- Returns `i128::MAX` when debt is zero.

---

## 5. Cross-Contract Call Patterns

### 5.1 Generated clients

All cross-contract calls use the generated client structs (via the
`soroban_sdk` contract impls), not raw `env.invoke_contract`.

```rust
use crate::ltoken::LTokenClient;       // from lending_pool's ltoken crate

let ltoken_client = LTokenClient::new(&env, &config.ltoken_address);
ltoken_client.mint(&depositor, &ltokens);
```

### 5.2 Producer → consumer dependency graph

```
PriceOracle  ──►  LendingPool  ──►  LiquidationEngine
InterestRateModel ──► ┘
LToken ──► LendingPool
```

- `LendingPool` reads prices from `PriceOracle` and rates from
  `InterestRateModel`.
- `LiquidationEngine` calls `LendingPool`, which calls back via
  `execute_liquidation` — the engine is the *only* caller authorized there.
- `LToken.mint`/`burn` accept the pool as the *only* caller.

### 5.3 Authorization

- User operations (`deposit`, `borrow`, `repay`, `withdraw`) require the
  relevant user `Address::require_auth()`.
- Admin operations (`add_asset`, `set_liquidation_engine`, `initialize`)
  require `admin.require_auth()`.
- `LToken.mint`/`burn` and `LendingPool.execute_liquidation` validate that
  `env.current_contract_address()` equals the stored authorized caller.

---

## 6. Frontend → Contract

The Next.js frontend talks to the contracts through:

1. `SorobanRpc.Server` (`@stellar/stellar-sdk`) configured in
   `src/lib/stellar.ts`. `submitTransaction` simulates → prepares → sends →
   polls for the result.
2. Generated TypeScript clients in `src/lib/contracts/` (regenerated by
   `scripts/generate_bindings.sh`).
3. React Query hooks in `src/hooks/`:
   - `useWallet` wraps the stellar-wallets-kit (Freighter modal).
   - `useLendingPool` wraps deposit/borrow/repay/withdraw mutations and the
     `get_health_factor` / `get_total_collateral_usd` / `get_total_debt_usd`
     queries.
   - `usePositions` reads per-asset user deposits and borrows.
   - `usePrices` polls the oracle every 30s.

The deploy sequence is fully scripted in `scripts/deploy_testnet.sh`.

---

## 7. Deploy & Wire-Up Order

Deployment order matters because each contract needs the previous ones'
addresses:

```
1. price_oracle            initialize(reflector_contract)
2. interest_rate_model     initialize(base_rate, slope1, slope2, optimal)
3. lending_pool            initialize(admin, oracle, irm)
4. ltoken (xlm)            initialize(pool, xlm_asset, "lXLM")
5. ltoken (usdc)           initialize(pool, usdc_asset, "lUSDC")
6. lending_pool            add_asset(...) x2
7. liquidation_engine      initialize(pool)
8. lending_pool            set_liquidation_engine(engine)
```

Each step is idempotent in `deploy_testnet.sh` — already-deployed IDs stored in
`.env` are reused on re-runs.

---

## 8. Security Model

- **Non-upgradeable** contracts in v1 — deploy-and-verify trust.
- **Checked arithmetic** — Soroban panics on overflow; `overflow-checks = true`
  in release profile.
- **Reentrancy** — mitigated by Soroban's single-threaded, deterministic
  host-execution model.
- **Oracle staleness** — prices older than `max_price_age` (300s) cause a
  panic, protecting against stale-price liquidations.
- **Over-collateralization** — each asset carries its own LTV / liquidation
  threshold; health factor must remain `>= 1.0` after every action.
- **Close factor** — at most 50% of a borrower's debt can be repaid in one
  liquidation, preventing griefing.

---

## 9. Audits & Next Steps

See the README's [Security](https://github.com/YOUR_GITHUB_USERNAME/lumenlend-protocol#security)
section for audit status. Before any mainnet deployment the codebase should
receive an independent audit, a bug-bounty launch, and expanded fuzz coverage —
tracked under the Phase 2 roadmap.