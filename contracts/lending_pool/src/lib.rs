#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

mod deposit;
mod math;
mod storage;
mod token;

use storage::DataKey;

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
        ltv: u32,               // e.g., 7500 = 75%
        liquidation_threshold: u32, // e.g., 8000 = 80%
        liquidation_bonus: u32,     // e.g., 500 = 5%
        reserve_factor: u32,        // e.g., 1000 = 10%
    );
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

    fn borrow(_env: Env, _borrower: Address, _asset: Address, _amount: i128) {
        panic!("LendingPool: not implemented");
    }

    fn repay(_env: Env, _repayer: Address, _asset: Address, _amount: i128) {
        panic!("LendingPool: not implemented");
    }

    fn withdraw(_env: Env, _withdrawer: Address, _asset: Address, _amount: i128) {
        panic!("LendingPool: not implemented");
    }

    fn get_health_factor(_env: Env, _user: Address) -> i128 {
        panic!("LendingPool: not implemented");
    }

    fn get_total_collateral_usd(_env: Env, _user: Address) -> i128 {
        0
    }

    fn get_total_debt_usd(_env: Env, _user: Address) -> i128 {
        0
    }

    fn add_asset(
        env: Env,
        asset: Address,
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
            // Placeholder until the per-asset lToken is deployed and registered.
            ltoken_address: Address::from_string(&String::from_str(
                &env,
                "GD5GCVMRXCLGELRFJECHWZ46Y3LIIFSJ3SLBWWY5MP33MOXSDTISRUKA",
            )),
            is_active: true,
        };
        storage::write_asset_config(&env, &asset, &config);
    }
}