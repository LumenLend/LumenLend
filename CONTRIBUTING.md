# Contributing to LumenLend

First off, thank you for considering contributing to LumenLend. This is a
community project built on the belief that a permissionless lending layer on
Stellar's Soroban platform benefits the entire ecosystem.

Reading and following these guidelines will help the maintainers process your
contribution faster and keep the codebase reviewable and safe. Every contract
ultimately handles real user funds: **correctness and auditability come before
speed.**

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Ask for Help](#how-to-ask-for-help)
- [Project Overview](#project-overview)
- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Project Layout](#project-layout)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Contributing Smart Contract Code (Rust)](#contributing-smart-contract-code-rust)
- [Contributing Frontend Code (TypeScript)](#contributing-frontend-code-typescript)
- [Writing Documentation](#writing-documentation)
- [Areas Where Help Is Needed](#areas-where-help-is-needed)
- [License](#license)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all
interactions with the project.

## How to Ask for Help

- **Bugs and feature requests** → open a GitHub issue.
- **Discussions** → the project's Telegram / Discord channels (see README).
- **Security vulnerabilities** → do **not** open a public issue. Follow the
  process in [SECURITY.md](SECURITY.md).

## Project Overview

LumenLend is a non-custodial lending protocol with five Soroban contracts:

| Contract | Description |
|---|---|
| `contracts/lending_pool` | Core pool: deposit, borrow, repay, withdraw, health factor |
| `contracts/ltoken` | SEP-41 receipt token minted to depositors |
| `contracts/interest_rate_model` | Utilization-based borrow/supply rates |
| `contracts/liquidation_engine` | Liquidates undercollateralized positions |
| `contracts/price_oracle` | Reflector oracle wrapper with staleness checks |

See [docs/architecture.md](docs/architecture.md) for the full system design.

## Prerequisites

- **Rust** (stable) + `wasm32-unknown-unknown` target
  ```bash
  rustup component add rustfmt clippy
  rustup target add wasm32-unknown-unknown
  ```
- **Soroban CLI** (`stellar`)
  ```bash
  cargo install --locked stellar-cli
  ```
- **Node.js** (v18+) and **pnpm** (v9+)
- Git

## Development Setup

```bash
# 1. Clone
git clone git@github.com:LumenLend/LumenLend.git
cd LumenLend

# 2. Rust contracts
cargo build --workspace
cargo test --workspace

# 3. Frontend
cd frontend
pnpm install
pnpm dev          # http://localhost:3000
pnpm build        # production build
pnpm lint
pnpm test
cd ..
```

> **Note on `pnpm install`:** the frontend ships a committed `pnpm-lock.yaml`
> so you can use `pnpm install --frozen-lockfile`. The `.npmrc` in
> `frontend/` disables pnpm's exotic-subdependencies block that some versions
> apply to a transitive git dependency of `@creit.tech/stellar-wallets-kit`.

### Environment

Copy `.env.example` to `.env` and fill in the network/contract values:

```bash
cp .env.example .env
```

All contract IDs are optional to run the UI locally; the frontend gracefully
falls back to placeholder values and shows data once a `.env` with deployed
`NEXT_PUBLIC_*` values is provided.

## Project Layout

```
contracts/     # Soroban smart contracts (Rust)
frontend/      # Next.js 14 app (TypeScript, Tailwind)
scripts/       # deploy_testnet.sh, generate_bindings.sh
docs/          # architecture, interest rate model, liquidation, risk, SCF
.github/       # workflows + templates
```

## Submitting a Pull Request

1. **Fork** the repository (or work on a feature branch in the org repo).
2. Create a focused branch:
   ```bash
   git checkout -b feat/your-feature-name
   # or fix/, docs/, chore/ prefixed branches
   ```
3. Make small, atomic commits with clear messages.
4. Add or update tests. Cover edge cases, not just happy paths.
5. Run the full verification suite (below) until green.
6. Open the PR with a clear description of **what** and **why**, and reference
   related issues.

**Checklist before opening a PR:**

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cd frontend && pnpm lint` passes
- [ ] `cd frontend && pnpm build` passes
- [ ] `cd frontend && pnpm test` passes
- [ ] New contract code has accompanying tests
- [ ] Docs updated if behavior changed
- [ ] No secrets, keys, or `.env` files committed

Your PR will be reviewed by at least one maintainer. Address review comments,
keep the conversation focused, and be patient — protocol code deserves thorough
review.

## Contributing Smart Contract Code (Rust)

- Follow Soroban SDK idioms: `#[contract]` / `#[contractimpl]`,
  `#[contracttype]` enums for storage keys, generated clients for
  cross-contract calls (no raw `env.invoke_contract`).
- **No `unwrap()` on fallible paths.** Use `checked_*` arithmetic followed by
  an explicit panic message, and `Option`/`Result` handling via `expect` with
  a descriptive message.
- All money math is 18-decimal `i128` fixed-point. Document the precision in
  the function doc-comment.
- Every entry-point change needs a test in `contracts/<crate>/tests/`.
  Edge cases to cover: zero amounts, empty storage, 0%/100% utilization, auth
  failures, and overflow paths.
- Non-upgradeable in v1: storage keys and function signatures are public API.
  Breaking changes must be discussed in an issue first.

## Contributing Frontend Code (TypeScript)

- Strict TypeScript; avoid `any` unless truly unavoidable.
- Use React Query (`@tanstack/react-query`) for all server/contract state.
- Follow existing Tailwind utility conventions; no inline style objects unless
  dynamic (e.g., health-factor bar width).
- All interactive elements need proper `aria` labels and accessible contrast.
- Handle loading, error, and empty states in every data-displaying component.
- Add Vitest + Testing Library tests alongside components.

## Writing Documentation

- Protocol docs live in `docs/` and are linked from the README.
- Prefer concrete formulas and worked examples over prose.
- When behavior changes, update the matching doc (`architecture.md`,
  `interest_rate_model.md`, `liquidation.md`, `risk_parameters.md`) in the
  same PR.

## Areas Where Help Is Needed

- 🧪 Additional test coverage: edge cases, fuzzing, oracle manipulation
  attack simulations.
- 🔮 Oracle staleness and circuit-breaker improvements.
- 🖥️ Frontend UX: health-factor previews, transaction monitoring, charts.
- 🤖 Liquidation bot reference implementation.
- 📝 Documentation and translations.
- 🎨 SCF grant application materials.

## License

By contributing you agree that your contributions are licensed under the
[MIT License](LICENSE).