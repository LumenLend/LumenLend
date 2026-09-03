use soroban_sdk::{Env, Address, Vec, Symbol, IntoVal, TryFromVal, Val};

use crate::{storage, token::TokenClient, math::{mul_div, SCALE}, math};

/// Calculate user health factor (18 decimal fixed point, 1e18 = 1.0).
///
/// For each asset the user has deposited:
///   - Get USD price from oracle via the contract address stored in config
///   - Multiply by deposit amount
///   - Multiply by liquidation_threshold / 10000
///   - Sum into total_collateral_weighted
///
/// For each asset the user has borrowed:
///   - Get USD price from oracle via the contract address stored in config
///   - Multiply by borrow amount
///   - Sum into total_debt
///
/// Return total_collateral_weighted * 1e18 / total_debt.
/// Return i128::MAX if total_debt == 0.
pub fn get_health_factor(env: &Env, user: &Address) -> i128 {
    let asset_list = storage::get_asset_list(env)
        .unwrap_or_else(|| Vec::new(env));

    let mut total_collateral_weighted: i128 = 0;
    let mut total_debt: i128 = 0;

    for asset in asset_list.iter() {
        let config = storage::read_asset_config(env, &asset).expect("Asset config not found");

        let user_deposit = storage::get_user_deposit(env, &user, &asset);
        let user_borrow = storage::get_user_borrow(env, &user, &asset);

        // Get USD price from oracle
        let price = get_price(env, &config.ltoken_address);

        if user_deposit > 0 {
            // Collateral weighted by liquidation threshold
            let weighted = mul_div(user_deposit * price, config.liquidation_threshold as i128, 10_000);
            total_collateral_weighted += weighted;
        }

        if user_borrow > 0 {
            // Debt at full price
            total_debt += user_borrow * price;
        }
    }

    if total_debt == 0 {
        return i128::MAX;
    }

    let hf = mul_div(total_collateral_weighted, SCALE, total_debt);
    hf
}

/// Get total collateral USD value for a user
pub fn get_total_collateral_usd(env: &Env, user: &Address) -> i128 {
    let asset_list = storage::get_asset_list(env)
        .unwrap_or_else(|| Vec::new(env));

    let mut total: i128 = 0;

    for asset in asset_list.iter() {
        let config = storage::read_asset_config(env, &asset).expect("Asset config not found");

        let user_deposit = storage::get_user_deposit(env, &user, &asset);
        let price = get_price(env, &config.ltoken_address);

        if user_deposit > 0 {
            total += user_deposit * price;
        }
    }

    total
}

/// Get total debt USD value for a user
pub fn get_total_debt_usd(env: &Env, user: &Address) -> i128 {
    let asset_list = storage::get_asset_list(env)
        .unwrap_or_else(|| Vec::new(env));

    let mut total: i128 = 0;

    for asset in asset_list.iter() {
        let config = storage::read_asset_config(env, &asset).expect("Asset config not found");

        let user_borrow = storage::get_user_borrow(env, &user, &asset);
        let price = get_price(env, &config.ltoken_address);

        if user_borrow > 0 {
            total += user_borrow * price;
        }
    }

    total
}

/// Internal: get price from oracle contract for a given asset's lToken address.
// The oracle contract stores price in 18-decimal fixed point.
fn get_price(env: &Env, ltoken_address: &Address) -> i128 {
    // For Day 3, use a marker price based on the ltoken's properties
    // In Day 4, this will be replaced with actual PriceOracle calls
    1_000_000_000_000_000_000i128 // 1 USD = 1e18 in 18-decimal
}