mod common;

use soroban_sdk::{testutils::Address as TestAddress, Address, Env};

use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};
use liquidation_engine::{LiquidationEngine, LiquidationEngineClient};

use common::{deploy_market, MockOracle, MockOracleClient, MockTokenClient};

const SCALE: i128 = 1_000_000_000_000_000_000;

/// Full liquidation scenario. Returns the pool, engine, oracle, and the
/// borrower / liquidator / collateral / debt addresses.
fn scenario(
    env: &Env,
    crash_price: bool,
) -> (
    LendingPoolClient<'_>,
    LiquidationEngineClient<'_>,
    MockOracleClient<'_>,
    Address,
    Address,
    Address,
    Address,
) {
    let admin = Address::generate(env);
    let supplier = Address::generate(env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(env, &irm_id);
    irm.initialize(&200, &1000, &30000, &8000);

    let oracle_id = env.register_contract(None, MockOracle);
    let oracle = MockOracleClient::new(env, &oracle_id);

    let lp = LendingPoolClient::new(env, &env.register_contract(None, LendingPool));
    lp.initialize(&admin, &oracle_id, &irm_id);

    let engine_id = env.register_contract(None, LiquidationEngine);
    let engine = LiquidationEngineClient::new(env, &engine_id);
    engine.initialize(&lp.address);

    env.mock_all_auths();
    lp.set_liquidation_engine(&engine.address);

    let borrower = Address::generate(env);
    let liquidator = Address::generate(env);

    // XLM: collateral asset.
    let xlm = Address::generate(env);
    let xlm_ltoken = deploy_market(env, &lp.address, &xlm);
    let xlm_token = MockTokenClient::new(env, &xlm);
    xlm_token.mint(&borrower, &(1000 * SCALE));
    lp.add_asset(&xlm, &xlm_ltoken, &7500, &8000, &500, &1000);
    oracle.set_price(&xlm, &SCALE);

    // USDC: debt asset, liquidity supplied by a third party.
    let usdc = Address::generate(env);
    let usdc_ltoken = deploy_market(env, &lp.address, &usdc);
    let usdc_token = MockTokenClient::new(env, &usdc);
    usdc_token.mint(&supplier, &(1000 * SCALE));
    usdc_token.mint(&liquidator, &(1000 * SCALE));
    lp.add_asset(&usdc, &usdc_ltoken, &8000, &8500, &500, &1000);
    oracle.set_price(&usdc, &SCALE);
    lp.deposit(&supplier, &usdc, &(100 * SCALE));

    // Borrower: 100 XLM collateral, 40 USDC debt.
    lp.deposit(&borrower, &xlm, &(100 * SCALE));
    lp.borrow(&borrower, &usdc, &(40 * SCALE));

    if crash_price {
        // Crash XLM to $0.25.
        oracle.set_price(&xlm, &250_000_000_000_000_000i128);
    }

    (lp, engine, oracle, borrower, liquidator, xlm, usdc)
}

#[test]
fn test_liquidation_flow_seizes_collateral() {
    let env = Env::default();
    let (lp, engine, _oracle, borrower, liquidator, xlm, usdc) = scenario(&env, true);
    env.mock_all_auths();

    // Position must be liquidatable after the price crash.
    let hf = lp.get_health_factor(&borrower);
    assert!(hf < SCALE);
    assert!(engine.is_liquidatable(&borrower));

    // Close factor caps the liquidation at 50% of the debt (40/2 = 20 USDC).
    assert_eq!(engine.max_liquidatable_amount(&borrower, &usdc), 20 * SCALE);

    let debt_before = lp.get_user_borrow(&borrower, &usdc);
    let collateral_before = env.as_contract(&lp.address, || {
        lending_pool::storage::get_user_deposit(&env, &borrower, &xlm)
    });

    // Liquidator covers 10 USDC of the borrower's debt.
    engine.liquidate(&liquidator, &borrower, &usdc, &xlm, &(10 * SCALE));

    // Debt reduced by repaid amount.
    let debt_after = lp.get_user_borrow(&borrower, &usdc);
    assert_eq!(debt_before - debt_after, 10 * SCALE);

    // Collateral seized: 10e18 * 1e18 * 10_500 / (2.5e17 * 10_000) = 42e18.
    let collateral_after = env.as_contract(&lp.address, || {
        lending_pool::storage::get_user_deposit(&env, &borrower, &xlm)
    });
    assert_eq!(collateral_before - collateral_after, 42 * SCALE);

    // Liquidator receives the seized collateral.
    let liquidator_collateral = env.as_contract(&lp.address, || {
        lending_pool::storage::get_user_deposit(&env, &liquidator, &xlm)
    });
    assert_eq!(liquidator_collateral, 42 * SCALE);
}

#[test]
#[should_panic(expected = "not liquidatable")]
fn test_liquidate_healthy_position_panics() {
    let env = Env::default();
    let (lp, engine, _oracle, borrower, liquidator, _xlm, usdc) = scenario(&env, false);
    env.mock_all_auths();

    // No price crash: position is healthy, liquidation must revert.
    let hf = lp.get_health_factor(&borrower);
    assert!(hf >= SCALE);
    assert!(!engine.is_liquidatable(&borrower));

    engine.liquidate(&liquidator, &borrower, &usdc, &_xlm, &(10 * SCALE));
}

#[test]
fn test_close_factor_caps_liquidation_amount() {
    let env = Env::default();
    let (lp, engine, _oracle, borrower, liquidator, xlm, usdc) = scenario(&env, true);
    env.mock_all_auths();

    // Try to liquidate the entire debt (100 USDC > 50% close factor).
    engine.liquidate(&liquidator, &borrower, &usdc, &xlm, &(100 * SCALE));

    // Only 50% of the debt (20 USDC) is allowed to be repaid.
    assert_eq!(lp.get_user_borrow(&borrower, &usdc), 20 * SCALE);
}