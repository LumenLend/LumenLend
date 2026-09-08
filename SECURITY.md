# Security Policy

LumenLend is a non-custodial lending protocol. Bugs in the smart contracts can
result in **loss of user funds**. We take all security reports seriously and
respond with priority.

## Reporting a Vulnerability

**Do not open a public GitHub issue** for security problems. Please report
privately:

1. Open a **private security advisory** on GitHub
   (Repository → **Security** → **Report a vulnerability**), or
2. Email the maintainers at `security@lumenlend.finance` (PGP key to be
   published before mainnet).

Please include:

- Which contracts/files are affected.
- A step-by-step description of the vulnerability, including any PoC code or a
  failing test harness (a Soroban test-env repro is ideal).
- Your assessment of impact (funds at risk, asset class, exploitability).
- Optional: your preferred disclosure/attribution preference.

### What happens next

- We will acknowledge your report within **48 hours**.
- We will work to validate and triage it, and communicate a fix timeline.
- We will not go public until we have either mitigated the issue or agreed on
  a disclosure schedule with you (90-day coordinated disclosure by default).

## Bug Bounty

A bug bounty program will be formally established **prior to mainnet launch**
with published payouts. Until then, for critical smart-contract vulnerabilities
that are verified and actionable, the LumenLend team will grant discretionary
rewards as funding allows.

## Scope

In scope:

- All contracts under `contracts/` (`lending_pool`, `ltoken`,
  `interest_rate_model`, `liquidation_engine`, `price_oracle`).
- The interaction logic between contracts (see `docs/architecture.md`).
- Oracle staleness / price-feed handling in `price_oracle`.

Out of scope (pre-mainnet):

- Frontend UI issues that do not affect funds.
- Purely theoretical issues without a concrete exploit path.
- Issues in third-party dependencies that are patched upstream.

## Disclosures

Verified vulnerabilities will be acknowledged in `SECURITY.md` after a fix is
shipped, unless the reporter requests anonymity.

## Best Practices for Contributors

- Never commit private keys, `.env` files, or testnet funds.
- When adding state-changing contract code, review the authorization model
  (`require_auth` and cross-contract caller checks) and the fixed-point
  arithmetic for overflow.
- Re-run the full test suite and `cargo clippy -- -D warnings` before opening
  a PR.