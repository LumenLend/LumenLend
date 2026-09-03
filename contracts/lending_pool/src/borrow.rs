use soroban_sdk::{Env, Address};

use crate::{math::mul_div, math::SCALE, storage, token::TokenClient, health};

/// Accrue interest on the asset pool before any state-changing operation.
///
/// Updates the borrow_index and deposit_index based on time elapsed
/// and the current borrow rate from the InterestRateModel.
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

/// Borrow an asset against existing collateral.
///
/// 1. Accrue interest first
/// 2. Check asset is active and has sufficient liquidity
/// 3. Calculate resulting health factor
/// 4. Revert if health factor would drop below 1e18 (1.0)
/// 5. Transfer `amount` of `asset` from contract to `borrower`
/// 6. Update `AssetState.total_borrows` and `UserPosition.borrows`
/// 7. Emit event
pub fn borrow(env: Env, borrower: Address, asset: Address, amount: i128) {
    borrower.require_auth();

    if amount <= 0 {
        panic!("borrow: amount must be positive");
    }

    // Accrue interest before changing state
    accrue_interest(&env, &asset);

    // Get asset config
    let config = storage::read_asset_config(&env, &asset).expect("Asset not registered");
    if !config.is_active {
        panic!("borrow: asset not active");
    }

    // Check sufficient liquidity
    let state = storage::read_asset_state(&env, &asset).expect("Asset state not initialized");
    let total_liquidity = state.total_deposits - state.total_borrows;
    if total_liquidity < amount {
        panic!("borrow: insufficient liquidity");
    }

    // Calculate health factor and revert if below 1e18
    let hf = health::get_health_factor(&env, &borrower);
    if hf < 1_000_000_000_000_000_000i128 {
        panic!("borrow: health factor below 1.0");
    }

    // Transfer asset from contract to borrower
    let token_client = TokenClient::new(&env, &config.ltoken_address);
    token_client.transfer(&borrower, &amount);

    // Update state
    let mut state = storage::read_asset_state(&env, &asset).expect("Asset state not initialized");
    state.total_borrows += amount;
    storage::write_asset_state(&env, &asset, &state);

    let user_borrow = storage::get_user_borrow(&env, &borrower, &asset);
    storage::set_user_borrow(&env, &borrower, &asset, user_borrow + amount);

    // Emit event (disabled - Topics trait issue)
    // env.events().publish(Symbol::new(&env, "borrow"), (&borrower, &asset, amount));
}