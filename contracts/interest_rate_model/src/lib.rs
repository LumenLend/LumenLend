//! InterestRateModel contract.
//!
//! Implements a kinked (two-slope) utilization-based interest rate model,
//! conceptually identical to Compound v2 / Aave v2:
//!
//! ```text
//! borrow_rate(util) =
//!     base_rate + slope1 * (util / optimal)                    if util <= optimal
//!     base_rate + slope1 + slope2 * ((util - optimal) / (1 - optimal))  otherwise
//!
//! supply_rate = borrow_rate * util * (1 - reserve_factor)
//! ```
//!
//! Configuration parameters are stored as basis points (`1/10000`), e.g.
//! `base_rate = 200` means 2% per year, `optimal_utilization = 8000` means 80%.
//! All returned rates are 18-decimal fixed point (`1e18` == 100%).

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env};

use crate::math::{from_basis_points, mul_div, SCALE};

mod math;

pub trait InterestRateModelTrait {
    /// Initialize the model parameters.
    ///
    /// - `base_rate`: minimum borrow rate at 0% utilization (e.g. 200 = 2%)
    /// - `slope1`: rate increase per unit of utilization below the kink
    ///   (e.g. 1000 = 10% at optimal utilization)
    /// - `slope2`: rate increase per unit of utilization above the kink
    ///   (e.g. 30000 = 300% at 100% utilization)
    /// - `optimal_utilization`: target utilization ratio (e.g. 8000 = 80%)
    fn initialize(env: Env, base_rate: u32, slope1: u32, slope2: u32, optimal_utilization: u32);

    /// Returns the annualized borrow rate (18-decimal fixed point).
    fn get_borrow_rate(env: Env, total_deposits: i128, total_borrows: i128) -> i128;

    /// Returns the annualized supply rate (18-decimal fixed point) after
    /// accounting for the protocol `reserve_factor` (basis points).
    fn get_supply_rate(
        env: Env,
        total_deposits: i128,
        total_borrows: i128,
        reserve_factor: u32,
    ) -> i128;

    /// Returns the utilization ratio (18-decimal fixed point), capped at 100%.
    fn get_utilization_rate(env: Env, total_deposits: i128, total_borrows: i128) -> i128;
}

#[contract]
pub struct InterestRateModel;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    BaseRate,
    Slope1,
    Slope2,
    OptimalUtilization,
}

#[contractimpl]
impl InterestRateModelTrait for InterestRateModel {
    fn initialize(env: Env, base_rate: u32, slope1: u32, slope2: u32, optimal_utilization: u32) {
        assert!(
            base_rate <= 10_000,
            "InterestRateModel: base rate exceeds 100%"
        );
        assert!(
            slope1 <= 1_000_000,
            "InterestRateModel: slope1 exceeds 10000%"
        );
        assert!(
            slope2 <= 1_000_000,
            "InterestRateModel: slope2 exceeds 10000%"
        );
        assert!(
            optimal_utilization <= 10_000,
            "InterestRateModel: optimal utilization exceeds 100%"
        );
        assert!(
            optimal_utilization > 0,
            "InterestRateModel: optimal utilization must be non-zero"
        );

        let storage = env.storage().instance();
        storage.set(&DataKey::BaseRate, &base_rate);
        storage.set(&DataKey::Slope1, &slope1);
        storage.set(&DataKey::Slope2, &slope2);
        storage.set(&DataKey::OptimalUtilization, &optimal_utilization);
    }

    fn get_utilization_rate(_env: Env, total_deposits: i128, total_borrows: i128) -> i128 {
        if total_deposits <= 0 || total_borrows <= 0 {
            return 0;
        }
        let utilization = mul_div(total_borrows, SCALE, total_deposits);
        if utilization > SCALE {
            SCALE
        } else {
            utilization
        }
    }

    fn get_borrow_rate(env: Env, total_deposits: i128, total_borrows: i128) -> i128 {
        let base_rate = from_basis_points(InterestRateModel::get_base_rate(&env));
        let slope1 = from_basis_points(InterestRateModel::get_slope1(&env));
        let slope2 = from_basis_points(InterestRateModel::get_slope2(&env));
        let optimal = from_basis_points(InterestRateModel::get_optimal_utilization(&env));

        let utilization = <InterestRateModel as InterestRateModelTrait>::get_utilization_rate(
            env.clone(),
            total_deposits,
            total_borrows,
        );

        if utilization <= optimal {
            // Below the kink: linear interpolation between base and base+slope1.
            let slope_part = mul_div(slope1, utilization, optimal);
            base_rate
                .checked_add(slope_part)
                .expect("InterestRateModel: borrow rate overflow")
        } else {
            // Above the kink: add slope2 scaled by the excess utilization.
            let excess = utilization
                .checked_sub(optimal)
                .expect("InterestRateModel: utilization below optimal");
            let one_minus_optimal = SCALE
                .checked_sub(optimal)
                .expect("InterestRateModel: optimal utilization at 100%");
            let slope_part = mul_div(slope2, excess, one_minus_optimal);
            base_rate
                .checked_add(slope1)
                .and_then(|rate| rate.checked_add(slope_part))
                .expect("InterestRateModel: borrow rate overflow")
        }
    }

    fn get_supply_rate(
        env: Env,
        total_deposits: i128,
        total_borrows: i128,
        reserve_factor: u32,
    ) -> i128 {
        assert!(
            reserve_factor <= 10_000,
            "InterestRateModel: reserve factor exceeds 100%"
        );

        let borrow_rate = <InterestRateModel as InterestRateModelTrait>::get_borrow_rate(
            env.clone(),
            total_deposits,
            total_borrows,
        );
        let utilization = <InterestRateModel as InterestRateModelTrait>::get_utilization_rate(
            env,
            total_deposits,
            total_borrows,
        );
        let reserve_factor_18 = from_basis_points(reserve_factor);
        let net = SCALE
            .checked_sub(reserve_factor_18)
            .expect("InterestRateModel: reserve factor at 100%");

        // supply = borrow_rate * utilization * (1 - reserve_factor)
        let partial = mul_div(borrow_rate, utilization, SCALE);
        mul_div(partial, net, SCALE)
    }
}

impl InterestRateModel {
    fn get_base_rate(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::BaseRate)
            .unwrap_or(0)
    }

    fn get_slope1(env: &Env) -> u32 {
        env.storage().instance().get(&DataKey::Slope1).unwrap_or(0)
    }

    fn get_slope2(env: &Env) -> u32 {
        env.storage().instance().get(&DataKey::Slope2).unwrap_or(0)
    }

    fn get_optimal_utilization(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::OptimalUtilization)
            .unwrap_or(0)
    }
}
