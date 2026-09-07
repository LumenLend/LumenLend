use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::{storage, math::{mul_div, SCALE}};

/// Calculate user health factor (18 decimal fixed point, 1e18 = 1.0).
///
/// For each asset the user has deposited:
///   - Get USD price from oracle
///   - Multiply by deposit amount
///   - Multiply by liquidation_threshold / 10000
///   - Sum into total_collateral_weighted
///
/// For each asset the user has borrowed:
///   - Get USD price from oracle
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

        let price = get_price(env, &asset);

        if user_deposit > 0 {
            let weighted = mul_div(user_deposit * price, config.liquidation_threshold as i128, 10_000);
            total_collateral_weighted += weighted;
        }

        if user_borrow > 0 {
            total_debt += user_borrow * price;
        }
    }

    if total_debt == 0 {
        return i128::MAX;
    }

    mul_div(total_collateral_weighted, SCALE, total_debt)
}

/// Get total collateral USD value for a user
pub fn get_total_collateral_usd(env: &Env, user: &Address) -> i128 {
    let asset_list = storage::get_asset_list(env)
        .unwrap_or_else(|| Vec::new(env));

    let mut total: i128 = 0;

    for asset in asset_list.iter() {
        let user_deposit = storage::get_user_deposit(env, &user, &asset);
        let price = get_price(env, &asset);

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
        let user_borrow = storage::get_user_borrow(env, &user, &asset);
        let price = get_price(env, &asset);

        if user_borrow > 0 {
            total += user_borrow * price;
        }
    }

    total
}

/// Internal: get the USD price of an asset (18-decimal fixed point).
///
/// Delegates to the configured PriceOracle via cross-contract call.
/// If the pool has not initialized an oracle, falls back to a marker
/// 1 USD price so tests remain deterministic.
pub fn get_price(env: &Env, asset: &Address) -> i128 {
    match storage::read_oracle(env) {
        Some(oracle) => {
            let args: Vec<Val> = (asset.clone(),).into_val(env);
            env.invoke_contract::<i128>(
                &oracle,
                &Symbol::new(env, "get_price"),
                args,
            )
        }
        None => 1_000_000_000_000_000_000i128, // 1 USD marker
    }
}

/// Execute a liquidation on behalf of the LiquidationEngine.
///
/// - Reduces `borrower`'s debt in `debt_asset` by `debt_amount`.
/// - Reduces `borrower`'s collateral in `collateral_asset` by `collateral_amount`.
/// - Credits `collateral_amount` of `collateral_asset` to `liquidator`.
/// - Updates the per-asset `AssetState` totals accordingly.
///
/// In this contract we track deposit and borrow balances in storage; actual
/// token transfers are performed by callers/frontend. For the on-chain
/// accounting, we move collateral from the borrower to the liquidator and
/// write off the repaid debt.
pub fn execute_liquidation(
    env: &Env,
    liquidator: &Address,
    borrower: &Address,
    debt_asset: &Address,
    collateral_asset: &Address,
    debt_amount: i128,
    collateral_amount: i128,
) {
    if debt_amount <= 0 {
        panic!("execute_liquidation: debt amount must be positive");
    }

    // Reduce borrower's debt in the debt asset.
    let borrower_debt = storage::get_user_borrow(env, borrower, debt_asset);
    if borrower_debt < debt_amount {
        panic!("execute_liquidation: debt exceeds borrower's outstanding borrow");
    }
    let new_borrower_debt = borrower_debt - debt_amount;
    storage::set_user_borrow(env, borrower, debt_asset, new_borrower_debt);

    // Reduce the pool's outstanding borrow in the debt asset.
    if let Some(mut state) = storage::read_asset_state(env, debt_asset) {
        state.total_borrows = state.total_borrows.saturating_sub(debt_amount);
        storage::write_asset_state(env, debt_asset, &state);
    }

    // Transfer collateral from borrower to liquidator.
    let borrower_collateral = storage::get_user_deposit(env, borrower, collateral_asset);
    if borrower_collateral < collateral_amount {
        panic!("execute_liquidation: collateral exceeds borrower's deposit");
    }
    let new_borrower_collateral = borrower_collateral - collateral_amount;
    storage::set_user_deposit(env, borrower, collateral_asset, new_borrower_collateral);

    let liquidator_collateral = storage::get_user_deposit(env, liquidator, collateral_asset);
    storage::set_user_deposit(
        env,
        liquidator,
        collateral_asset,
        liquidator_collateral + collateral_amount,
    );
}
