//! Integration tests for the InterestRateModel contract.
//!
//! Default model under test: `base_rate = 200` (2%), `slope1 = 1000` (10%),
//! `slope2 = 30000` (300%), `optimal_utilization = 8000` (80%).

use interest_rate_model::{InterestRateModel, InterestRateModelClient};
use soroban_sdk::Env;

/// 18-decimal fixed-point scale (`1.0`).
const ONE: i128 = 1_000_000_000_000_000_000;

/// Expected values for the default model (18-decimal fixed point).
const BASE_RATE: i128 = 200 * ONE / 10_000; // 0.02
const SLOPE1_RATE: i128 = 1000 * ONE / 10_000; // 0.10
const SLOPE2_RATE: i128 = 30000 * ONE / 10_000; // 3.00
const OPTIMAL_RATE: i128 = 8000 * ONE / 10_000; // 0.80

const BASE_RATE_AT_OPTIMAL: i128 = BASE_RATE + SLOPE1_RATE; // 0.12
const BASE_RATE_AT_100: i128 = BASE_RATE + SLOPE1_RATE + SLOPE2_RATE; // 3.12

/// Registers the model with default parameters and hands the initialized
/// client to `f`. The `Env` must outlive the client, so it is kept alive for
/// the duration of the closure.
fn with_model<F: FnOnce(&InterestRateModelClient)>(f: F) {
    let env = Env::default();
    let contract_id = env.register_contract(None, InterestRateModel);
    let client = InterestRateModelClient::new(&env, &contract_id);
    client.initialize(&200, &1000, &30000, &8000);
    f(&client);
}

#[test]
fn test_initialize_stores_parameters() {
    with_model(|client| {
        // 0% utilization must produce exactly the base rate...
        assert_eq!(client.get_borrow_rate(&1_000, &0), BASE_RATE);
        // ...and 80% utilization (at the kink) must produce base_rate + slope1.
        assert_eq!(client.get_borrow_rate(&1_000, &800), BASE_RATE_AT_OPTIMAL);
        // At 100% utilization the borrow rate must reach base + slope1 + slope2.
        assert_eq!(client.get_borrow_rate(&1_000, &1_000), BASE_RATE_AT_100);
    });
}

#[test]
#[should_panic(expected = "optimal utilization must be non-zero")]
fn test_initialize_rejects_zero_optimal_utilization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InterestRateModel);
    let client = InterestRateModelClient::new(&env, &contract_id);
    client.initialize(&200, &1000, &30000, &0);
}

#[test]
#[should_panic(expected = "base rate exceeds 100%")]
fn test_initialize_rejects_base_rate_above_100_percent() {
    let env = Env::default();
    let contract_id = env.register_contract(None, InterestRateModel);
    let client = InterestRateModelClient::new(&env, &contract_id);
    client.initialize(&10_001, &1000, &30000, &8000);
}

#[test]
fn test_utilization_rate_zero_borrows_is_zero() {
    with_model(|client| {
        assert_eq!(client.get_utilization_rate(&1_000, &0), 0);
    });
}

#[test]
fn test_utilization_rate_zero_deposits_is_zero() {
    with_model(|client| {
        // total_deposits == 0 must not panic and must yield 0% utilization.
        assert_eq!(client.get_utilization_rate(&0, &500), 0);
    });
}

#[test]
fn test_utilization_rate_fifty_percent() {
    with_model(|client| {
        assert_eq!(client.get_utilization_rate(&1_000, &500), 50 * ONE / 100);
    });
}

#[test]
fn test_utilization_rate_at_optimal() {
    with_model(|client| {
        assert_eq!(client.get_utilization_rate(&1_000, &800), OPTIMAL_RATE);
    });
}

#[test]
fn test_utilization_rate_one_hundred_percent() {
    with_model(|client| {
        assert_eq!(client.get_utilization_rate(&1_000, &1_000), ONE);
    });
}

#[test]
fn test_utilization_rate_capped_at_100_percent() {
    with_model(|client| {
        // Borrows above deposits must be clamped to 100% utilization.
        assert_eq!(client.get_utilization_rate(&1_000, &1_200), ONE);
    });
}

#[test]
fn test_borrow_rate_zero_utilization_is_base_rate() {
    with_model(|client| {
        assert_eq!(client.get_borrow_rate(&1_000, &0), BASE_RATE);
    });
}

#[test]
fn test_borrow_rate_at_optimal_is_base_plus_slope1() {
    with_model(|client| {
        assert_eq!(client.get_borrow_rate(&1_000, &800), BASE_RATE_AT_OPTIMAL);
    });
}

#[test]
fn test_borrow_rate_below_optimal_is_linear() {
    with_model(|client| {
        // 50% utilization => 2% + 10% * (50/80) = 8.25%.
        let expected = BASE_RATE + SLOPE1_RATE * 50 / 80;
        assert_eq!(client.get_borrow_rate(&1_000, &500), expected);
    });
}

#[test]
fn test_borrow_rate_above_optimal_uses_slope2() {
    with_model(|client| {
        // 90% utilization => 2% + 10% + 300% * (10/20) = 162%.
        let expected = BASE_RATE_AT_OPTIMAL + SLOPE2_RATE * 10 / 20;
        assert_eq!(client.get_borrow_rate(&1_000, &900), expected);
    });
}

#[test]
fn test_borrow_rate_at_100_percent_is_maximum() {
    with_model(|client| {
        assert_eq!(client.get_borrow_rate(&1_000, &1_000), BASE_RATE_AT_100);
    });
}

#[test]
fn test_borrow_rate_zero_deposits_does_not_panic() {
    with_model(|client| {
        // No deposits means no borrows can exist; utilization is 0%, so the
        // borrow rate falls back to the base rate instead of panicking.
        assert_eq!(client.get_borrow_rate(&0, &0), BASE_RATE);
    });
}

#[test]
fn test_supply_rate_zero_utilization_is_zero() {
    with_model(|client| {
        assert_eq!(client.get_supply_rate(&1_000, &0, &1000), 0);
    });
}

#[test]
fn test_supply_rate_at_optimal_with_reserve_factor() {
    with_model(|client| {
        // 12% borrow rate * 80% utilization * (1 - 10% reserve) = 8.64%.
        let expected =
            BASE_RATE_AT_OPTIMAL * OPTIMAL_RATE / ONE * (ONE - 1000 * ONE / 10_000) / ONE;
        assert_eq!(client.get_supply_rate(&1_000, &800, &1000), expected);
    });
}

#[test]
fn test_supply_rate_no_reserve_factor() {
    with_model(|client| {
        // 12% borrow rate * 80% utilization * (1 - 0) = 9.6%.
        let expected = BASE_RATE_AT_OPTIMAL * OPTIMAL_RATE / ONE;
        assert_eq!(client.get_supply_rate(&1_000, &800, &0), expected);
    });
}

#[test]
fn test_supply_rate_zero_deposits_does_not_panic() {
    with_model(|client| {
        assert_eq!(client.get_supply_rate(&0, &0, &1000), 0);
    });
}

#[test]
fn test_supply_rate_full_reserve_factor_is_zero() {
    with_model(|client| {
        // 100% reserve factor means all interest is retained by the protocol.
        assert_eq!(client.get_supply_rate(&1_000, &800, &10_000), 0);
    });
}
