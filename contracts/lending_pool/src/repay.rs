use soroban_sdk::{Env, Address};

use crate::{math::mul_div, math::SCALE, storage, token::TokenClient};

/// Repay a borrow position (partial or full).
///
/// 1. Accrue interest first
/// 2. Cap repay amount at outstanding borrow (no over-repayment)
/// 3. Transfer `amount` from `repayer` to contract
/// 4. Update `AssetState.total_borrows` and `UserPosition.borrows`
/// 5. If borrow reaches 0, clean up storage entry
/// 6. Emit event
pub fn repay(env: Env, repayer: Address, asset: Address, amount: i128) {
    repayer.require_auth();

    if amount <= 0 {
        panic!("repay: amount must be positive");
    }

    // Accrue interest first
    accrue_interest(&env, &asset);

    // Get asset config and state
    let config = storage::read_asset_config(&env, &asset).expect("Asset not registered");
    let mut state = storage::read_asset_state(&env, &asset).expect("Asset state not initialized");

    // Get user's current borrow and cap at outstanding amount
    let user_borrow = storage::get_user_borrow(&env, &repayer, &asset);
    let repay_amount = amount.min(user_borrow);

    if repay_amount == 0 {
        panic!("repay: no outstanding borrow to repay");
    }

    // Transfer asset from repayer to contract
    let token_client = TokenClient::new(&env, &config.ltoken_address);
    token_client.transfer_from(&env.current_contract_address(), &repayer, &amount);

    // Update state
    state.total_borrows -= repay_amount;
    storage::write_asset_state(&env, &asset, &state);

    let user_new_borrow = user_borrow - repay_amount;
    if user_new_borrow == 0 {
        // Clean up storage entry if borrow is fully repaid
        storage::set_user_borrow(&env, &repayer, &asset, 0);
    } else {
        storage::set_user_borrow(&env, &repayer, &asset, user_new_borrow);
    }

    // Emit event (disabled - Topics trait issue)
    // env.events().publish(Symbol::new(&env, "repay"), (&repayer, &asset, repay_amount));
}

/// Internal interest accrual function (shared with deposit and borrow)
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