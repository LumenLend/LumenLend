#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

pub mod borrow;
pub mod deposit;
pub mod health;
pub mod math;
pub mod repay;
pub mod storage;
pub mod token;
pub mod withdraw;

use storage::{read_asset_config, DataKey};

#[contract]
pub struct LendingPool;

pub trait LendingPoolTrait {
    /// Initialize the pool with supported assets and config.
    fn initialize(
        env: Env,
        admin: Address,
        oracle: Address,
        interest_rate_model: Address,
    );

    /// Deposit an asset into the lending pool.
    /// Mints lTokens to the depositor.
    fn deposit(env: Env, depositor: Address, asset: Address, amount: i128);

    /// Borrow an asset against existing collateral.
    fn borrow(env: Env, borrower: Address, asset: Address, amount: i128);

    /// Repay a borrow position (partial or full).
    fn repay(env: Env, repayer: Address, asset: Address, amount: i128);

    /// Withdraw deposited collateral (up to available liquidity).
    fn withdraw(env: Env, withdrawer: Address, asset: Address, amount: i128);

    /// Returns user health factor (18 decimal fixed point, 1e18 = 1.0).
    fn get_health_factor(env: Env, user: Address) -> i128;

    /// Returns user's total collateral value in USD (18 decimal).
    fn get_total_collateral_usd(env: Env, user: Address) -> i128;

    /// Returns user's total debt value in USD (18 decimal).
    fn get_total_debt_usd(env: Env, user: Address) -> i128;

    /// Add a new supported asset (admin only).
    fn add_asset(
        env: Env,
        asset: Address,
        ltoken_address: Address,
        ltv: u32,               // e.g., 7500 = 75%
        liquidation_threshold: u32, // e.g., 8000 = 80%
        liquidation_bonus: u32,     // e.g., 500 = 5%
        reserve_factor: u32,        // e.g., 1000 = 10%
    );

    /// Set the LiquidationEngine contract address (admin only).
    fn set_liquidation_engine(env: Env, engine: Address);

    /// Execute a liquidation: repays borrower debt and transfers collateral
    /// to the liquidator. Callable only by the registered LiquidationEngine.
    fn execute_liquidation(
        env: Env,
        liquidator: Address,
        borrower: Address,
        debt_asset: Address,
        collateral_asset: Address,
        debt_amount: i128,
        collateral_amount: i128,
    );

    /// Returns a user's outstanding borrow for an asset (18 decimal).
    fn get_user_borrow(env: Env, user: Address, asset: Address) -> i128;

    /// Returns the USD price of an asset in 18-decimal fixed-point by
    /// delegating to the configured PriceOracle.
    fn get_asset_price(env: Env, asset: Address) -> i128;

    /// Returns the liquidation bonus (basis points) for an asset.
    fn get_asset_liquidation_bonus(env: Env, asset: Address) -> u32;
}

#[contractimpl]
impl LendingPoolTrait for LendingPool {
    fn initialize(
        env: Env,
        admin: Address,
        oracle: Address,
        interest_rate_model: Address,
    ) {
        let storage = env.storage().instance();
        if storage.has(&DataKey::Admin) {
            panic!("LendingPool: already initialized");
        }
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Oracle, &oracle);
        storage.set(&DataKey::InterestRateModel, &interest_rate_model);
    }

    fn deposit(env: Env, depositor: Address, asset: Address, amount: i128) {
        deposit::deposit(env, depositor, asset, amount)
    }

    fn borrow(env: Env, borrower: Address, asset: Address, amount: i128) {
        borrow::borrow(env, borrower, asset, amount)
    }

    fn repay(env: Env, repayer: Address, asset: Address, amount: i128) {
        repay::repay(env, repayer, asset, amount)
    }

    fn withdraw(env: Env, withdrawer: Address, asset: Address, amount: i128) {
        withdraw::withdraw(env, withdrawer, asset, amount)
    }

    fn get_health_factor(env: Env, user: Address) -> i128 {
        health::get_health_factor(&env, &user)
    }

    fn get_total_collateral_usd(env: Env, user: Address) -> i128 {
        health::get_total_collateral_usd(&env, &user)
    }

    fn get_total_debt_usd(env: Env, user: Address) -> i128 {
        health::get_total_debt_usd(&env, &user)
    }

    fn add_asset(
        env: Env,
        asset: Address,
        ltoken_address: Address,
        ltv: u32,
        liquidation_threshold: u32,
        liquidation_bonus: u32,
        reserve_factor: u32,
    ) {
        // Admin-only authorization.
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("LendingPool: not initialized");
        admin.require_auth();

        // Asset must not already be registered.
        if env.storage().instance().has(&DataKey::AssetConfig(asset.clone())) {
            panic!("LendingPool: asset already registered");
        }

        let config = storage::AssetConfig {
            ltv,
            liquidation_threshold,
            liquidation_bonus,
            reserve_factor,
            ltoken_address,
            is_active: true,
        };
        storage::write_asset_config(&env, &asset, &config);

        // Initialize asset state if not present
        if !env.storage().instance().has(&DataKey::AssetState(asset.clone())) {
            let state = storage::AssetState {
                total_deposits: 0,
                total_borrows: 0,
                last_update_timestamp: env.ledger().timestamp(),
                borrow_index: math::SCALE,
                deposit_index: math::SCALE,
            };
            storage::write_asset_state(&env, &asset, &state);
        }

        // Add to asset list
        storage::add_to_asset_list(&env, &asset);
    }

    fn set_liquidation_engine(env: Env, engine: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("LendingPool: not initialized");
        admin.require_auth();
        storage::write_liquidation_engine(&env, &engine);
    }

    fn execute_liquidation(
        env: Env,
        liquidator: Address,
        borrower: Address,
        debt_asset: Address,
        collateral_asset: Address,
        debt_amount: i128,
        collateral_amount: i128,
    ) {
        // Only the registered LiquidationEngine may invoke this.
        let engine: Address = env
            .storage()
            .instance()
            .get(&DataKey::LiquidationEngine)
            .expect("LendingPool: liquidation engine not set");
        engine.require_auth();
        liquidator.require_auth();

        health::execute_liquidation(
            &env,
            &liquidator,
            &borrower,
            &debt_asset,
            &collateral_asset,
            debt_amount,
            collateral_amount,
        );
    }

    fn get_user_borrow(env: Env, user: Address, asset: Address) -> i128 {
        storage::get_user_borrow(&env, &user, &asset)
    }

    fn get_asset_price(env: Env, asset: Address) -> i128 {
        health::get_price(&env, &asset)
    }

    fn get_asset_liquidation_bonus(env: Env, asset: Address) -> u32 {
        let config = read_asset_config(&env, &asset).expect("Asset not registered");
        config.liquidation_bonus
    }
}
