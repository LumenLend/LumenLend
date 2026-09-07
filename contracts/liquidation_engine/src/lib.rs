#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Symbol, Vec};

pub mod calculator;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    LendingPool,
}

const SCALE: i128 = 1_000_000_000_000_000_000;

pub trait LiquidationEngineTrait {
    fn initialize(env: Env, lending_pool: Address);

    fn liquidate(
        env: Env,
        liquidator: Address,
        borrower: Address,
        debt_asset: Address,
        collateral_asset: Address,
        debt_amount: i128,
    );

    fn is_liquidatable(env: Env, borrower: Address) -> bool;

    fn max_liquidatable_amount(env: Env, borrower: Address, debt_asset: Address) -> i128;
}

#[contract]
pub struct LiquidationEngine;

#[contractimpl]
impl LiquidationEngineTrait for LiquidationEngine {
    fn initialize(env: Env, lending_pool: Address) {
        let storage = env.storage().instance();
        if storage.has(&DataKey::LendingPool) {
            panic!("LiquidationEngine: already initialized");
        }
        storage.set(&DataKey::LendingPool, &lending_pool);
    }

    fn liquidate(
        env: Env,
        liquidator: Address,
        borrower: Address,
        debt_asset: Address,
        collateral_asset: Address,
        debt_amount: i128,
    ) {
        liquidator.require_auth();

        if debt_amount <= 0 {
            panic!("LiquidationEngine: debt amount must be positive");
        }

        let lp: Address = env
            .storage()
            .instance()
            .get(&DataKey::LendingPool)
            .expect("LiquidationEngine: not initialized");

        // 1. Verify the position is liquidatable (health factor < 1e18)
        let hf = Self::invoke_health_factor(&env, &lp, &borrower);
        if hf >= SCALE {
            panic!("LiquidationEngine: position is not liquidatable");
        }

        // 2. Cap debt at the close factor
        let outstanding = Self::invoke_get_user_borrow(&env, &lp, &borrower, &debt_asset);
        let max = calculator::apply_close_factor(outstanding);
        let actual_debt = if debt_amount > max { max } else { debt_amount };
        if actual_debt <= 0 {
            panic!("LiquidationEngine: no liquidatable debt");
        }

        // 3. Fetch prices from the lending pool's oracle
        let debt_price = Self::invoke_get_asset_price(&env, &lp, &debt_asset);
        let collateral_price = Self::invoke_get_asset_price(&env, &lp, &collateral_asset);

        // 4. Fetch the liquidation bonus for the collateral asset
        let bonus = Self::invoke_get_asset_liquidation_bonus(&env, &lp, &collateral_asset);

        // 5. Calculate collateral to seize
        let collateral_to_seize = calculator::calculate_collateral_to_seize(
            actual_debt,
            debt_price,
            collateral_price,
            bonus,
        );

        // 6. Call lending pool to execute the actual token transfers
        Self::invoke_execute_liquidation(
            &env,
            &lp,
            &liquidator,
            &borrower,
            &debt_asset,
            &collateral_asset,
            &actual_debt,
            &collateral_to_seize,
        );

        // 7. Emit event
        env.events().publish(
            (Symbol::new(&env, "liquidation"), liquidator, borrower),
            actual_debt,
        );
    }

    fn is_liquidatable(env: Env, borrower: Address) -> bool {
        let lp: Address = env
            .storage()
            .instance()
            .get(&DataKey::LendingPool)
            .expect("LiquidationEngine: not initialized");
        Self::invoke_health_factor(&env, &lp, &borrower) < SCALE
    }

    fn max_liquidatable_amount(env: Env, borrower: Address, debt_asset: Address) -> i128 {
        let lp: Address = env
            .storage()
            .instance()
            .get(&DataKey::LendingPool)
            .expect("LiquidationEngine: not initialized");
        let outstanding = Self::invoke_get_user_borrow(&env, &lp, &borrower, &debt_asset);
        calculator::apply_close_factor(outstanding)
    }
}

impl LiquidationEngine {
    fn invoke_health_factor(env: &Env, lp: &Address, user: &Address) -> i128 {
        let args: Vec<soroban_sdk::Val> = (user.clone(),).into_val(env);
        env.invoke_contract::<i128>(lp, &Symbol::new(env, "get_health_factor"), args)
    }

    fn invoke_get_user_borrow(
        env: &Env,
        lp: &Address,
        user: &Address,
        asset: &Address,
    ) -> i128 {
        let args: Vec<soroban_sdk::Val> = (user.clone(), asset.clone()).into_val(env);
        env.invoke_contract::<i128>(lp, &Symbol::new(env, "get_user_borrow"), args)
    }

    fn invoke_get_asset_price(env: &Env, lp: &Address, asset: &Address) -> i128 {
        let args: Vec<soroban_sdk::Val> = (asset.clone(),).into_val(env);
        env.invoke_contract::<i128>(lp, &Symbol::new(env, "get_asset_price"), args)
    }

    fn invoke_get_asset_liquidation_bonus(
        env: &Env,
        lp: &Address,
        asset: &Address,
    ) -> u32 {
        let args: Vec<soroban_sdk::Val> = (asset.clone(),).into_val(env);
        env.invoke_contract::<u32>(lp, &Symbol::new(env, "get_asset_liquidation_bonus"), args)
    }

    fn invoke_execute_liquidation(
        env: &Env,
        lp: &Address,
        liquidator: &Address,
        borrower: &Address,
        debt_asset: &Address,
        collateral_asset: &Address,
        debt_amount: &i128,
        collateral_amount: &i128,
    ) {
        let args: Vec<soroban_sdk::Val> = (
            liquidator.clone(),
            borrower.clone(),
            debt_asset.clone(),
            collateral_asset.clone(),
            *debt_amount,
            *collateral_amount,
        )
            .into_val(env);
        env.invoke_contract::<()>(lp, &Symbol::new(env, "execute_liquidation"), args);
    }
}
