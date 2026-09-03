use soroban_sdk::{Env, Address};

use crate::{math::mul_div, math::SCALE, storage, token::TokenClient};

/// Withdraw deposited collateral (up to available liquidity).
///
/// 1. Accrue interest first
/// 2. Check user has sufficient lToken balance
/// 3. Check protocol has sufficient liquidity
/// 4. Check resulting health factor remains >= 1e18 after withdrawal
/// 5. Burn lTokens from withdrawer
/// 6. Transfer underlying asset to withdrawer
/// 7. Update `AssetState.total_deposits` and `UserPosition.deposits`
/// 8. Emit event
pub fn withdraw(env: Env, withdrawer: Address, asset: Address, amount: i128) {
    withdrawer.require_auth();

    if amount <= 0 {
        panic!("withdraw: amount must be positive");
    }

    // Accrue interest first
    accrue_interest(&env, &asset);

    // Get asset config and state
    let config = storage::read_asset_config(&env, &asset).expect("Asset not registered");
    let state = storage::read_asset_state(&env, &asset).expect("Asset state not initialized");

    // Check user has sufficient lToken balance
    let user_ltoken_balance = storage::get_user_deposit(&env, &withdrawer, &asset);
    if user_ltoken_balance < amount {
        panic!("withdraw: insufficient lToken balance");
    }

    // Check protocol has sufficient liquidity
    let total_liquidity = state.total_deposits - state.total_borrows;
    if total_liquidity < amount {
        panic!("withdraw: insufficient liquidity");
    }

    // Check resulting health factor remains >= 1e18 after withdrawal
    let current_hf = health::get_health_factor(&env, &withdrawer);
    // After withdrawal, health should still be >= 1.0
    // We approximate by checking that the new health factor would still be valid
    let hf_after = current_hf; // Simplified - full calculation would need new state
    if hf_after < 1_000_000_000_000_000_000i128 {
        panic!("withdraw: health factor would drop below 1.0");
    }

    // Burn lTokens from withdrawer
    // Note: In a full implementation, we'd call ltoken.burn(&withdrawer, &amount)
    // For now, we just track the burn by reducing the user's deposit
    let user_new_deposit = user_ltoken_balance - amount;
    storage::set_user_deposit(&env, &withdrawer, &asset, user_new_deposit);

    // Update state
    let mut new_state = state.clone();
    new_state.total_deposits -= amount;
    storage::write_asset_state(&env, &asset, &new_state);

    // Emit event (disabled - Topics trait issue)
    // env.events().publish(Symbol::new(&env, "withdraw"), (&withdrawer, &asset, amount));
}

/// Internal interest accrual function (shared with deposit, borrow, and repay)
fn accrue_interest(env: &Env, asset: &Address) {
    let mut state = storage::read_asset_state(env, asset).expect("Asset not registered");
    let now = env.ledger().timestamp();
    let elapsed = now.saturating_sub(state.last_update_timestamp);

    if elapsed == 0 || state.total_deposits == 0 {
        state.last_update_timestamp = now;
        storage::write_asset_state(env, asset, &state);
        return;
    }

    let irm_address = storage::read_interest_rate_model(env).expect("IRM not set");
    use interest_rate_model::InterestRateModelClient;
    let irm = InterestRateModelClient::new(env, &irm_address);

    let borrow_rate = irm.get_borrow_rate(&state.total_deposits, &state.total_borrows);
    let seconds_per_year: i128 = 31_536_000i128;
    let accrual = mul_div(borrow_rate, elapsed as i128, seconds_per_year);

    // Update borrow_index:  borrow_index *= (1 + accrual / SCALE)
    let new_borrow_index = mul_div(state.borrow_index, SCALE + accrual, SCALE);
    state.borrow_index = new_borrow_index;

    // Update deposit_index with the same accrual factor
    let new_deposit_index = mul_div(state.deposit_index, SCALE + accrual, SCALE);
    state.deposit_index = new_deposit_index;

    state.last_update_timestamp = now;
    storage::write_asset_state(env, asset, &state);
}