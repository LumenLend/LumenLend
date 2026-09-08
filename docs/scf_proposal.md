# Stellar Community Fund (SCF) Proposal — LumenLend

> **Status:** Draft — this document is the working SCF grant application for
> LumenLend. It is a living document and will be finalized before submission.

## Executive Summary

LumenLend brings the first serious, audited, **permissionless lending layer to
Soroban**. Stellar processes billions in payments and hosts a thriving
stablecoin ecosystem, yet builders and holders have no composable on-chain
lending primitive to put that liquidity to work. LumenLend fills that gap with
a non-custodial borrow/lend protocol — deposits earn yield, collateral unlocks
leverage, and an open liquidation market keeps the system solvent.

## Problem

- **Idle capital.** XLM, USDC, and EURC holders on Stellar cannot earn yield
  without moving off-chain.
- **No leverage.** Ecosystem teams building on Soroban cannot borrow against
  their assets — a missing foundational building block for nearly every DeFi
  app (leverage, yield strategies, structured products).
- **DeFi gap.** Every mature ecosystem (Ethereum, Solana, Cosmos) has a
  lending primitive; Stellar is the outlier.
- **Ecosystem immaturity.** Institutional players evaluating Stellar observe
  the missing DeFi money-primitive layer.

## Solution

1. A **permissionless lending pool** — deposit and earn variable APY.
2. **Collateralized borrowing** — borrow against deposited assets within
   per-asset LTV bounds.
3. A **utilization-based interest rate model** (kinked, Compound/Aave-style)
   that prices risk dynamically and protects liquidity.
4. An **open liquidation market** with per-asset thresholds and bonuses.
5. **lTokens (SEP-41)** — composable, interest-bearing receipt tokens.

## Why It Works on Stellar

- Fast finality (~5s) and sub-cent fees make frequent lending/liquidation
  interactions viable.
- Native asset support (XLM is network-native; USDC/EURC are ubiquitous).
- The **Reflector** decentralized oracle network provides the price
  infrastructure required for a safe lending protocol — a chance to ship an
  on-chain lending primitive without bootstrapping oracles from scratch.
- Soroban's Rust/Sorosense deterministic execution model reduces the attack
  surface versus general-purpose VMs.

## Architecture (Summary)

See [architecture.md](architecture.md) for full detail.

```
LendingPool ──► LToken          LendingPool ──► PriceOracle (Reflector)
    │                ▲               │
    └──► InterestRateModel  LiquidationEngine ──► LendingPool.execute_liquidation
```

- Contract split isolates risk: price logic, rate logic, receipt tokens, and
  the liquidation engine are separate, individually upgradeable-only-by-redeploy.
- Cross-contract calls use Soroban's typed contract-interface system.
- All math is 18-decimal fixed point with checked arithmetic.

## Roadmap & Milestones

Phase 1 (this grant):
- [x] Workspace, `InterestRateModel`, `LToken`, and core `LendingPool`
      (deposit/borrow/repay/withdraw + health factor)
- [x] `PriceOracle` (Reflector wrapper) and `LiquidationEngine`
- [x] Integration test suite across all contracts
- [x] Frontend MVP (markets, supply, borrow, positions)
- [x] Testnet deployment scripts and TypeScript bindings
- [ ] Independent security audit (pre-mainnet gate)
- [ ] Bug bounty launch
- [ ] Frontend test coverage ≥ 80%, expanded fuzz/edge-case tests
- [ ] Mainnet deployment

Phase 2 (post-audit):
- Governance/DAO migration path
- Flash loans, additional assets (EURC, wBTC/wETH via bridge)
- Liquidation bot reference implementation
- Institutional permissioned pools

## Funding Request

TBD — to be filled with a defined milestone breakdown. Requested funds cover
audit costs, testnet rewards, and sustained maintenance.

## Impact

- First permissionless lending primitive on Soroban → unlocks the ecosystem's
  DeFi flywheel (leverage, structured products, yield aggregators).
- Composability via SEP-41 lTokens.
- Reference security-hardening practices for the emerging Soroban DeFi stack
  (staleness checks, close factors, over-collateralization).

## Team & Contact

TBD — maintainers will be listed here at submission time. Contact:
[Telegram](https://t.me/lumenlend) · [Twitter](https://twitter.com/lumenlend)