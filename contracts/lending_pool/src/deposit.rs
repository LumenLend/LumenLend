use soroban_sdk::{Address, Env, Symbol};

use crate::{math::{mul_div, SCALE}, storage, token::TokenClient};

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

/// Transfer `amount` of `asset` from `depositor` to the contract,
/// accrue interest first, then mint lTokens.
pub fn deposit(env: Env, depositor: Address, asset: Address, amount: i128) {
    depositor.require_auth();

    if amount <= 0 {
        panic!("deposit: amount must be positive");
    }

    // Accrue interest before changing state
    accrue_interest(&env, &asset);

    // Get asset config
    let config = storage::read_asset_config(&env, &asset).expect("Asset not registered");
    if !config.is_active {
        panic!("deposit: asset not active");
    }

    // Transfer asset from depositor to lending pool
    let token_client = TokenClient::new(&env, &asset);
    token_client.transfer_from(&depositor, &depositor, &env.current_contract_address(), &amount);

    // Calculate lTokens to mint and mint them
    use ltoken::LTokenClient;
    let ltoken_client = LTokenClient::new(&env, &config.ltoken_address);
    let exchange_rate = ltoken_client.exchange_rate();
    let ltoken_amount = (amount * 1_000_000_000_000_000_000i128) / exchange_rate;

    ltoken_client.mint(&depositor, &ltoken_amount);

    // Update state
    let mut state = storage::read_asset_state(&env, &asset).expect("Asset not registered");
    state.total_deposits += amount;
    storage::write_asset_state(&env, &asset, &state);

    let user_deposit = storage::get_user_deposit(&env, &depositor, &asset);
    storage::set_user_deposit(&env, &depositor, &asset, user_deposit + amount);

    // Emit event (disabled - Topics trait issue)
    // env.events().publish(Symbol::new(&env, "deposit"), (&depositor, &asset, amount));
}
