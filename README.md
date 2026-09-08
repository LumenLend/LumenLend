# LumenLend

> A decentralized, non-custodial lending and borrowing protocol built on Stellar's Soroban smart contract platform.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built on Stellar](https://img.shields.io/badge/Built%20on-Stellar-blueviolet)](https://stellar.org)
[![Powered by Soroban](https://img.shields.io/badge/Powered%20by-Soroban-orange)](https://soroban.stellar.org)
[![SCF Applicant](https://img.shields.io/badge/SCF-Applicant-brightgreen)](https://communityfund.stellar.org)

---

## Table of Contents

- [Overview](#overview)
- [Problem Statement](#problem-statement)
- [Solution](#solution)
- [Features](#features)
- [Architecture](#architecture)
  - [Smart Contracts](#smart-contracts)
  - [Contract Interactions](#contract-interactions)
  - [Interest Rate Model](#interest-rate-model)
  - [Liquidation Mechanism](#liquidation-mechanism)
- [Tech Stack](#tech-stack)
- [Repository Structure](#repository-structure)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Environment Setup](#environment-setup)
  - [Running Tests](#running-tests)
  - [Deploying to Testnet](#deploying-to-testnet)
  - [Deploying to Mainnet](#deploying-to-mainnet)
- [Smart Contract Reference](#smart-contract-reference)
  - [LendingPool](#lendingpool)
  - [LToken (Receipt Token)](#ltoken-receipt-token)
  - [InterestRateModel](#interestratemodel)
  - [LiquidationEngine](#liquidationengine)
  - [PriceOracle](#priceoracle)
- [Frontend](#frontend)
- [Security](#security)
  - [Audit Status](#audit-status)
  - [Known Risks](#known-risks)
  - [Bug Bounty](#bug-bounty)
- [Roadmap](#roadmap)
- [SCF Grant Proposal](#scf-grant-proposal)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

---

## Overview

**LumenLend** is a fully on-chain, non-custodial lending and borrowing protocol deployed on [Soroban](https://soroban.stellar.org) — Stellar's smart contract platform. It enables users to deposit supported Stellar assets as collateral, borrow against them, earn interest as a lender, and participate in a transparent liquidation market.

LumenLend is inspired by the architecture of Aave and Compound but is purpose-built for the Stellar ecosystem — leveraging Stellar's fast finality, low fees, native asset support, and the Reflector decentralized oracle network for real-time price feeds.

---

## Problem Statement

The Stellar network processes billions of dollars in payments and remittances annually and hosts a thriving stablecoin ecosystem (USDC, EURC, and others). Yet despite this liquidity, **there is no mature, audited, decentralized lending protocol on Soroban**.

This creates several pain points:

- **Idle capital**: Stellar asset holders cannot put their XLM, USDC, or other tokens to productive use by earning yield.
- **No leverage**: Traders and builders cannot borrow against existing holdings to access liquidity without selling.
- **DeFi gap**: Developers building on Stellar cannot compose with a lending primitive — a foundational building block for virtually every DeFi application (leverage, yield strategies, structured products).
- **Ecosystem immaturity**: The absence of a lending layer signals incompleteness to institutional players and developers evaluating Stellar for DeFi.

---

## Solution

LumenLend fills this gap by providing:

1. A **permissionless lending pool** where anyone can deposit assets and earn interest.
2. A **collateralized borrowing system** where depositors can borrow other assets against their collateral.
3. A **utilization-based variable interest rate model** that algorithmically prices risk.
4. An **open liquidation market** that incentivizes third parties to keep the protocol solvent.
5. A **receipt token system (lTokens)** that represent depositor positions and can be composed with other protocols.

---

## Features

### Core Protocol
- 💰 **Supply & Earn** — Deposit XLM, USDC, EURC, or other supported assets to earn variable APY
- 🏦 **Borrow Against Collateral** — Use deposited assets as collateral to borrow other assets
- 📈 **Variable Interest Rates** — Rates automatically adjust based on pool utilization
- ⚡ **Fast & Cheap** — Soroban transactions settle in ~5 seconds with sub-cent fees
- 🔁 **lToken Receipt System** — Receive interest-bearing lTokens when depositing (e.g., lXLM, lUSDC)
- 🩺 **Health Factor Monitoring** — Real-time position health tracking to prevent unexpected liquidations

### Risk Management
- 🔒 **Overcollateralization** — All loans require collateral above the Loan-to-Value (LTV) threshold
- ⚠️ **Liquidation Engine** — Undercollateralized positions can be liquidated by anyone for a bonus
- 🔮 **Reflector Oracle Integration** — Real-time, decentralized price feeds via [Reflector Protocol](https://reflector.network)
- 📊 **Per-Asset Risk Parameters** — Each asset has individually configured LTV, liquidation threshold, and liquidation bonus

### Developer & Ecosystem
- 🧩 **Composable** — lTokens and the lending pool are designed for composability with other Soroban protocols
- 🔓 **Open Source** — Fully open source under MIT license
- 🧪 **Comprehensive Test Suite** — Unit and integration tests using the Soroban test framework
- 📝 **Fully Documented** — NatSpec-style inline documentation on all contracts

---

## Architecture

### Smart Contracts

LumenLend is composed of five core contracts:

```
contracts/
├── lending_pool/        # Core protocol logic: deposit, borrow, repay, withdraw
├── ltoken/              # ERC20-style receipt token for depositors
├── interest_rate_model/ # Utilization-based interest rate calculations
├── liquidation_engine/  # Handles undercollateralized position liquidation
└── price_oracle/        # Wrapper around Reflector oracle for price feeds
```

### Contract Interactions

```
                        ┌──────────────────────────┐
                        │     User / Frontend       │
                        │  (Freighter Wallet)        │
                        └────────────┬─────────────┘
                                     │
                        ┌────────────▼─────────────┐
                        │      LendingPool          │
                        │  (Core Protocol Entry)    │
                        │                           │
                        │  deposit()                │
                        │  borrow()                 │
                        │  repay()                  │
                        │  withdraw()               │
                        │  get_health_factor()      │
                        └──┬──────────┬─────────────┘
                           │          │
             ┌─────────────▼──┐  ┌────▼──────────────┐
             │  LToken        │  │  InterestRateModel │
             │  (per asset)   │  │                    │
             │                │  │  get_borrow_rate() │
             │  mint()        │  │  get_supply_rate() │
             │  burn()        │  │  utilization_rate()│
             │  balance_of()  │  └────────────────────┘
             └────────────────┘
                           │
             ┌─────────────▼──────────┐   ┌────────────────────┐
             │  LiquidationEngine     │   │  PriceOracle       │
             │                        │   │  (Reflector Wrap)  │
             │  liquidate()           │◄──│                    │
             │  is_liquidatable()     │   │  get_price()       │
             │  calculate_bonus()     │   │  get_prices()      │
             └────────────────────────┘   └────────────────────┘
```

### Interest Rate Model

LumenLend uses a **kinked (two-slope) interest rate model**, identical in concept to Compound v2 and Aave v2. The model is designed to keep utilization in an optimal range.

```
Borrow APR
    │
    │                              /
    │                             /  Slope 2 (steep)
    │                            /
    │                           /
    │               ┌──────────* Kink (Optimal Utilization ~80%)
    │              /
    │             /  Slope 1 (gradual)
    │────────────/
    │ Base Rate
    └──────────────────────────────────── Utilization %
    0%                80%               100%
```

**Variables:**
- `base_rate`: Minimum borrow rate at 0% utilization (e.g., 2% APR)
- `slope1`: Rate increase per 1% utilization below kink (e.g., 0.1x)
- `slope2`: Rate increase per 1% utilization above kink (e.g., 3x) — discourages over-utilization
- `optimal_utilization`: The target utilization ratio (e.g., 80%)

**Supply APR** = `Borrow APR × Utilization Rate × (1 - Reserve Factor)`

### Liquidation Mechanism

When a borrower's **health factor drops below 1.0**, their position becomes eligible for liquidation.

```
Health Factor = (Collateral Value × Liquidation Threshold) / Total Borrow Value

Health Factor < 1.0  →  Position is undercollateralized  →  Liquidation eligible
```

**Liquidation flow:**
1. Liquidator calls `liquidate(borrower, debt_asset, collateral_asset, amount)`
2. Protocol verifies health factor < 1.0
3. Liquidator repays up to 50% of the borrower's debt (close factor)
4. Liquidator receives collateral equal to repaid debt value + **liquidation bonus** (e.g., 5–10%)
5. Borrower's health factor is restored above 1.0

This creates a self-sustaining, incentivized liquidation market — no bot infrastructure is required at the protocol level.

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Smart Contracts | Rust + Soroban SDK | Core protocol logic |
| Oracle | Reflector Protocol | Decentralized price feeds |
| Frontend | Next.js 14 + TypeScript | User interface |
| Wallet Integration | Freighter SDK | Stellar wallet connector |
| Stellar Interaction | `@stellar/stellar-sdk` | Blockchain interaction |
| Soroban RPC | Soroban RPC (Horizon) | Contract invocation |
| Styling | Tailwind CSS | UI styling |
| Testing (Contracts) | Soroban test env (Rust) | Contract unit/integration tests |
| Testing (Frontend) | Vitest + Testing Library | Frontend tests |
| CI/CD | GitHub Actions | Automated testing & deployment |
| Package Manager | pnpm | Frontend dependency management |

---

## Repository Structure

```
lumenlend-protocol/
│
├── contracts/                        # All Soroban smart contracts (Rust)
│   ├── lending_pool/
│   │   ├── src/
│   │   │   ├── lib.rs                # Contract entry point & trait definition
│   │   │   ├── storage.rs            # Persistent storage keys and helpers
│   │   │   ├── deposit.rs            # Deposit logic
│   │   │   ├── borrow.rs             # Borrow logic
│   │   │   ├── repay.rs              # Repayment logic
│   │   │   ├── withdraw.rs           # Withdrawal logic
│   │   │   └── health.rs             # Health factor calculation
│   │   ├── Cargo.toml
│   │   └── tests/
│   │       ├── deposit_tests.rs
│   │       ├── borrow_tests.rs
│   │       ├── repay_tests.rs
│   │       └── liquidation_tests.rs
│   │
│   ├── ltoken/
│   │   ├── src/
│   │   │   ├── lib.rs                # lToken contract (SEP-41 token interface)
│   │   │   ├── mint.rs
│   │   │   └── burn.rs
│   │   └── Cargo.toml
│   │
│   ├── interest_rate_model/
│   │   ├── src/
│   │   │   ├── lib.rs                # Interest rate model contract
│   │   │   └── math.rs               # Fixed-point math utilities
│   │   └── Cargo.toml
│   │
│   ├── liquidation_engine/
│   │   ├── src/
│   │   │   ├── lib.rs                # Liquidation engine contract
│   │   │   └── calculator.rs         # Bonus and close factor calculations
│   │   └── Cargo.toml
│   │
│   └── price_oracle/
│       ├── src/
│       │   ├── lib.rs                # Reflector oracle wrapper
│       │   └── feed.rs               # Price feed parsing
│       └── Cargo.toml
│
├── frontend/                         # Next.js frontend application
│   ├── src/
│   │   ├── app/                      # Next.js App Router pages
│   │   │   ├── page.tsx              # Dashboard / Markets overview
│   │   │   ├── supply/page.tsx       # Supply/Deposit page
│   │   │   ├── borrow/page.tsx       # Borrow page
│   │   │   └── positions/page.tsx    # User positions page
│   │   ├── components/               # Reusable UI components
│   │   │   ├── WalletConnect.tsx
│   │   │   ├── MarketTable.tsx
│   │   │   ├── PositionCard.tsx
│   │   │   ├── HealthFactorBar.tsx
│   │   │   └── TransactionModal.tsx
│   │   ├── hooks/                    # Custom React hooks
│   │   │   ├── useWallet.ts
│   │   │   ├── useLendingPool.ts
│   │   │   ├── usePositions.ts
│   │   │   └── usePrices.ts
│   │   ├── lib/                      # Soroban contract clients & utilities
│   │   │   ├── contracts/            # Auto-generated contract bindings
│   │   │   ├── stellar.ts            # Stellar SDK setup
│   │   │   └── constants.ts          # Contract addresses, asset configs
│   │   └── types/                    # TypeScript type definitions
│   ├── public/
│   ├── package.json
│   ├── tailwind.config.ts
│   └── tsconfig.json
│
├── scripts/                          # Deployment and utility scripts
│   ├── deploy_testnet.sh
│   ├── deploy_mainnet.sh
│   ├── initialize_pool.sh
│   └── generate_bindings.sh          # Generate TS bindings from contracts
│
├── docs/                             # Extended documentation
│   ├── architecture.md
│   ├── interest_rate_model.md
│   ├── liquidation.md
│   ├── risk_parameters.md
│   └── scf_proposal.md
│
├── audits/                           # Security audit reports
│   └── .gitkeep
│
├── .github/
│   └── workflows/
│       ├── test.yml                  # CI: run tests on every PR
│       └── deploy.yml                # CD: deploy frontend on merge to main
│
├── Cargo.toml                        # Workspace Cargo manifest
├── Cargo.lock
├── .env.example                      # Environment variable template
├── .gitignore
├── LICENSE
└── README.md
```

---

## Getting Started

### Prerequisites

Ensure you have the following installed:

```bash
# Rust (stable toolchain + wasm32 target)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Soroban CLI
cargo install --locked soroban-cli

# Node.js (v18+) and pnpm
curl -fsSL https://fnm.vercel.app/install | bash
fnm use 18
npm install -g pnpm

# Stellar accounts for testnet (optional — scripts handle this)
soroban keys generate --global testnet-deployer --network testnet
```

Verify your installation:

```bash
soroban --version   # Should output: soroban 20.x.x or higher
rustc --version     # Should output: rustc 1.7x.x or higher
node --version      # Should output: v18.x.x or higher
pnpm --version      # Should output: 8.x.x or higher
```

### Installation

```bash
# Clone the repository
git clone https://github.com/YOUR_GITHUB_USERNAME/lumenlend-protocol.git
cd lumenlend-protocol

# Install Rust contract dependencies
cargo build

# Install frontend dependencies
cd frontend
pnpm install
cd ..
```

### Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Configure Stellar network (testnet by default)
# Edit .env with your values:
```

```env
# .env

# Network configuration
STELLAR_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Deployer keypair (NEVER commit real secret keys)
DEPLOYER_SECRET_KEY=S...

# Reflector oracle contract address (testnet)
REFLECTOR_CONTRACT_ID=C...

# Deployed contract addresses (filled after deployment)
LENDING_POOL_CONTRACT_ID=
LTOKEN_XLM_CONTRACT_ID=
LTOKEN_USDC_CONTRACT_ID=
INTEREST_RATE_MODEL_CONTRACT_ID=
LIQUIDATION_ENGINE_CONTRACT_ID=
PRICE_ORACLE_CONTRACT_ID=

# Frontend
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_LENDING_POOL_CONTRACT_ID=
```

### Running Tests

```bash
# Run all contract tests
cargo test

# Run tests for a specific contract
cargo test -p lending-pool

# Run tests with output (useful for debugging)
cargo test -- --nocapture

# Run frontend tests
cd frontend
pnpm test

# Run frontend tests with coverage
pnpm test:coverage
```

Expected output:

```
running 24 tests
test deposit::test_deposit_xlm ... ok
test deposit::test_deposit_usdc ... ok
test deposit::test_deposit_zero_amount_fails ... ok
test borrow::test_borrow_within_ltv ... ok
test borrow::test_borrow_exceeds_ltv_fails ... ok
test borrow::test_borrow_interest_accrual ... ok
test repay::test_full_repayment ... ok
test repay::test_partial_repayment ... ok
test liquidation::test_liquidation_below_health_factor ... ok
test liquidation::test_liquidation_above_health_factor_fails ... ok
...
test result: ok. 24 passed; 0 failed; 0 ignored
```

### Deploying to Testnet

```bash
# Fund your testnet deployer account
soroban keys fund testnet-deployer --network testnet

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Run the deployment script
chmod +x scripts/deploy_testnet.sh
./scripts/deploy_testnet.sh

# Generate TypeScript bindings for the frontend
chmod +x scripts/generate_bindings.sh
./scripts/generate_bindings.sh

# Start the frontend in development mode
cd frontend
pnpm dev
```

The frontend will be available at `http://localhost:3000`.

### Deploying to Mainnet

> ⚠️ **Mainnet deployment requires an audited codebase. Do not deploy unaudited contracts with real user funds.**

```bash
# Ensure STELLAR_NETWORK=mainnet in .env
# Ensure DEPLOYER_SECRET_KEY is your mainnet key (handle with extreme care)

chmod +x scripts/deploy_mainnet.sh
./scripts/deploy_mainnet.sh
```

---

## Smart Contract Reference

### LendingPool

The central contract. All user interactions flow through this contract.

**Interface:**

```rust
pub trait LendingPoolTrait {
    /// Initialize the pool with supported assets and config
    fn initialize(
        env: Env,
        admin: Address,
        oracle: Address,
        interest_rate_model: Address,
    );

    /// Deposit an asset into the lending pool
    /// Mints lTokens to the depositor
    fn deposit(env: Env, depositor: Address, asset: Address, amount: i128);

    /// Borrow an asset against existing collateral
    fn borrow(env: Env, borrower: Address, asset: Address, amount: i128);

    /// Repay a borrow position (partial or full)
    fn repay(env: Env, repayer: Address, asset: Address, amount: i128);

    /// Withdraw deposited collateral (up to available liquidity)
    fn withdraw(env: Env, withdrawer: Address, asset: Address, amount: i128);

    /// Returns user health factor (18 decimal fixed point, 1e18 = 1.0)
    fn get_health_factor(env: Env, user: Address) -> i128;

    /// Returns user's total collateral value in USD (18 decimal)
    fn get_total_collateral_usd(env: Env, user: Address) -> i128;

    /// Returns user's total debt value in USD (18 decimal)
    fn get_total_debt_usd(env: Env, user: Address) -> i128;

    /// Add a new supported asset (admin only)
    fn add_asset(
        env: Env,
        asset: Address,
        ltv: u32,               // e.g., 7500 = 75%
        liquidation_threshold: u32, // e.g., 8000 = 80%
        liquidation_bonus: u32,     // e.g., 500 = 5%
        reserve_factor: u32,        // e.g., 1000 = 10%
    );
}
```

**Key storage:**
- `UserPosition` — tracks each user's deposits and borrows per asset
- `AssetConfig` — stores risk parameters per supported asset
- `AssetState` — tracks total deposits, borrows, and accrued interest per asset

---

### LToken (Receipt Token)

An interest-bearing token issued to depositors. Follows the [SEP-41 token interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md).

- **lXLM** — issued for XLM deposits
- **lUSDC** — issued for USDC deposits
- **Exchange rate** increases over time as interest accrues, meaning 1 lToken redeems for more of the underlying asset over time.

```rust
pub trait LTokenTrait {
    fn initialize(env: Env, lending_pool: Address, underlying_asset: Address, name: String, symbol: String);
    fn mint(env: Env, to: Address, amount: i128);         // callable by LendingPool only
    fn burn(env: Env, from: Address, amount: i128);       // callable by LendingPool only
    fn balance(env: Env, owner: Address) -> i128;
    fn total_supply(env: Env) -> i128;
    fn underlying_asset(env: Env) -> Address;
    fn exchange_rate(env: Env) -> i128;                   // lToken → underlying
}
```

---

### InterestRateModel

Calculates borrow and supply rates based on pool utilization.

```rust
pub trait InterestRateModelTrait {
    fn initialize(
        env: Env,
        base_rate: u32,            // e.g., 200 = 2% per year
        slope1: u32,               // e.g., 1000 = 10% at optimal
        slope2: u32,               // e.g., 30000 = 300% at 100% utilization
        optimal_utilization: u32,  // e.g., 8000 = 80%
    );

    /// Returns annualized borrow rate (18 decimal fixed point)
    fn get_borrow_rate(env: Env, total_deposits: i128, total_borrows: i128) -> i128;

    /// Returns annualized supply rate (18 decimal fixed point)
    fn get_supply_rate(
        env: Env,
        total_deposits: i128,
        total_borrows: i128,
        reserve_factor: u32,
    ) -> i128;

    /// Returns utilization ratio (18 decimal fixed point)
    fn get_utilization_rate(env: Env, total_deposits: i128, total_borrows: i128) -> i128;
}
```

---

### LiquidationEngine

Handles liquidation of undercollateralized positions.

```rust
pub trait LiquidationEngineTrait {
    /// Liquidate an undercollateralized borrower
    /// Liquidator repays debt_amount of debt_asset
    /// Liquidator receives collateral_asset + bonus
    fn liquidate(
        env: Env,
        liquidator: Address,
        borrower: Address,
        debt_asset: Address,
        collateral_asset: Address,
        debt_amount: i128,
    );

    /// Returns true if the position is liquidatable (health factor < 1.0)
    fn is_liquidatable(env: Env, borrower: Address) -> bool;

    /// Returns the maximum amount a liquidator can repay (close factor applied)
    fn max_liquidatable_amount(env: Env, borrower: Address, debt_asset: Address) -> i128;
}
```

**Close Factor:** Maximum 50% of a borrower's outstanding debt can be liquidated in a single transaction, preventing "liquidation griefing."

---

### PriceOracle

A wrapper around the [Reflector Protocol](https://reflector.network) oracle that provides price feeds for all supported assets.

```rust
pub trait PriceOracleTrait {
    fn initialize(env: Env, reflector_contract: Address);

    /// Returns the USD price of an asset (18 decimal fixed point)
    fn get_price(env: Env, asset: Address) -> i128;

    /// Returns USD prices for multiple assets in a single call
    fn get_prices(env: Env, assets: Vec<Address>) -> Vec<i128>;

    /// Returns the timestamp of the last price update
    fn last_updated(env: Env, asset: Address) -> u64;
}
```

> Prices are validated for staleness. If a price feed is older than the configured `max_price_age` (default: 5 minutes), the transaction will fail to prevent oracle manipulation attacks.

---

## Frontend

The LumenLend frontend is a Next.js 14 application using the App Router, Tailwind CSS, and the Freighter wallet.

**Key pages:**
- `/` — Markets dashboard showing all supported assets, supply APY, borrow APR, and total liquidity
- `/supply` — Deposit assets into the protocol
- `/borrow` — Borrow against your collateral with real-time health factor preview
- `/positions` — View and manage your open positions, monitor health factor

**Wallet support:**
- [Freighter](https://freighter.app) — Primary Stellar browser extension wallet
- [xBull Wallet](https://xbull.app) — Secondary support

**Running locally:**

```bash
cd frontend
pnpm dev       # Development mode with hot reload
pnpm build     # Production build
pnpm start     # Start production server
pnpm lint      # Run ESLint
pnpm test      # Run Vitest tests
```

---

## Security

Security is the top priority for any lending protocol. LumenLend follows these practices:

### Audit Status

| Version | Auditor | Status | Report |
|---|---|---|---|
| v1.0.0 | TBD | Planned (pre-mainnet) | - |

> If you are an auditor interested in reviewing LumenLend, please reach out via [GitHub Issues](../../issues).

### Known Risks

All DeFi protocols carry inherent risks. Users should understand the following before interacting with LumenLend:

| Risk | Description | Mitigation |
|---|---|---|
| Smart Contract Risk | Bugs in contract code could result in loss of funds | Comprehensive tests; external audit before mainnet |
| Oracle Risk | Stale or manipulated price feeds could trigger incorrect liquidations | Reflector decentralized oracle; staleness checks; circuit breakers |
| Liquidation Risk | Rapid price drops could result in bad debt if liquidations don't execute in time | Conservative LTV ratios; incentivized liquidation bonuses |
| Utilization Risk | If utilization reaches 100%, withdrawals may be temporarily blocked | Steep slope2 in interest rate model discourages over-utilization |
| Governance Risk | Admin key controls critical parameters before DAO launch | Multisig admin from day one; planned DAO migration |

### Security Measures

- **No upgradeable contracts** in v1 (immutability by default for trust)
- **Admin multisig** required for all privileged operations
- **Reentrancy protection** enforced via Soroban's inherent execution model
- **Integer overflow** impossible — Soroban uses checked arithmetic
- **Access control** on all privileged functions via `require_auth()`
- **Staleness checks** on all oracle price reads

### Bug Bounty

A bug bounty program will be established prior to mainnet launch. Responsible disclosure: see [**SECURITY.md**](SECURITY.md) and open a **private** security advisory on GitHub.

---

## Roadmap

### Phase 1 — Foundation (Months 1–3)
- [x] Project architecture and specification
- [ ] Core contracts: LendingPool, lToken, InterestRateModel
- [ ] Liquidation engine
- [ ] Reflector oracle integration
- [ ] Full test suite (>90% coverage)
- [ ] Testnet deployment
- [ ] Frontend MVP (supply, borrow, positions)
- [ ] SCF grant application submission

### Phase 2 — Hardening (Months 4–6)
- [ ] Independent security audit
- [ ] Bug bounty program launch
- [ ] Frontend polish and UX improvements
- [ ] Liquidation bot reference implementation
- [ ] Mainnet deployment
- [ ] Protocol documentation site

### Phase 3 — Growth (Months 7–12)
- [ ] Governance token design and distribution
- [ ] DAO migration for protocol parameter control
- [ ] Flash loan support
- [ ] Additional asset markets (EURC, wBTC, wETH via bridge)
- [ ] Cross-protocol composability integrations
- [ ] Yield strategy vaults built on top of LumenLend

### Phase 4 — Ecosystem (Year 2+)
- [ ] Institutional lending markets (permissioned pools)
- [ ] Fixed-rate lending tranches
- [ ] Cross-chain collateral (via Stellar bridge)
- [ ] Mobile app (React Native)

---

## Contributing

Contributions are welcome! LumenLend is an open-source protocol and community participation is encouraged.

Please read the full guide in [**CONTRIBUTING.md**](CONTRIBUTING.md), and note all participants must follow our [**Code of Conduct**](CODE_OF_CONDUCT.md). If you believe you have found a security issue, review [**SECURITY.md**](SECURITY.md) **before** opening an issue.

**Protocol documentation:**
- [Architecture](docs/architecture.md)
- [Interest Rate Model](docs/interest_rate_model.md)
- [Liquidation Mechanism](docs/liquidation.md)
- [Risk Parameters](docs/risk_parameters.md)
- [SCF Grant Proposal](docs/scf_proposal.md)

### How to Contribute

1. **Fork** the repository
2. **Create a feature branch**: `git checkout -b feature/your-feature-name`
3. **Write tests** for any new functionality
4. **Ensure all tests pass**: `cargo test && cd frontend && pnpm test`
5. **Submit a Pull Request** with a clear description of changes

### Contribution Guidelines

- All smart contract changes require accompanying tests
- Follow Rust idioms and the existing code style (`cargo fmt` and `cargo clippy` must pass)
- Frontend code must pass ESLint (`pnpm lint`)
- Security-sensitive changes will receive extra scrutiny and may require additional review
- Breaking changes to contract interfaces must be discussed in an issue first

### Areas Where Help Is Needed

- 🧪 Additional test coverage (edge cases, fuzz testing)
- 🔮 Oracle price manipulation attack simulations
- 🖥️ Frontend UI/UX improvements
- 📝 Documentation improvements
- 🌍 Translations of documentation
- 🤖 Liquidation bot implementation

---

## License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

---

## Acknowledgements

- [Reflector Protocol](https://reflector.network) — for the decentralized oracle infrastructure
- [Aave](https://aave.com) and [Compound](https://compound.finance) — for pioneering the lending protocol model that inspires this architecture
- [Soroban Documentation](https://soroban.stellar.org/docs) — comprehensive developer resources
- [Freighter Wallet](https://freighter.app) — for the Stellar browser wallet SDK
- The Stellar Community Fund — for supporting open-source ecosystem development

---

<div align="center">
  <strong>Built with ❤️ for the Stellar ecosystem</strong>
  <br />
  <a href="https://t.me/lumenlend">Telegram</a> ·
  <a href="https://twitter.com/lumenlend">Twitter</a> ·
  <a href="https://discord.gg/lumenlend">Discord</a> ·
  <a href="https://lumenlend.finance">Website</a>
</div>
