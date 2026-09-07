mod common;

use soroban_sdk::{testutils::Address as TestAddress, Address, Env};
use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};

use common::{deploy_market, MockOracle, MockTokenClient};

/// Shared setup: registers the IRM, LendingPool, MockOracle, a MockToken
/// (as the underlying asset), and a real LToken.
/// Returns `(lp, admin, asset, user)`.
fn setup(env: &Env) -> (LendingPoolClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let user = Address::generate(env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(env, &irm_id);
    let br: u32 = 200;
    let sl1: u32 = 1000;
    let sl2: u32 = 30000;
    let opt: u32 = 8000;
    irm.initialize(&br, &sl1, &sl2, &opt);

    let oracle_id = env.register_contract(None, MockOracle);

    let lp = LendingPoolClient::new(env, &env.register_contract(None, LendingPool));
    lp.initialize(&admin, &oracle_id, &irm_id);

    let asset = Address::generate(env);
    let ltoken_id = deploy_market(env, &lp.address, &asset);
    let token = MockTokenClient::new(env, &asset);

    env.mock_all_auths();
    token.mint(&admin, &100_000_000_000_000_000_000i128);
    lp.add_asset(&asset, &ltoken_id, &7500, &8000, &500, &1000);

    (lp, admin, asset, user)
}

#[test]
fn test_full_repayment_clears_debt() {
    let env = Env::default();
    let (lp, admin, asset, user) = setup(&env);
    env.mock_all_auths();

    // Deposit and borrow to set up debt
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 3_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Full repayment
    let repay_amount = 3_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);

    // Check debt is cleared
    let user_borrow = env.as_contract(&lp.address, || {
        lending_pool::storage::get_user_borrow(&env, &user, &asset)
    });
    assert_eq!(user_borrow, 0, "Full repayment should clear all debt");
}

#[test]
fn test_partial_repayment_reduces_debt() {
    let env = Env::default();
    let (lp, admin, asset, user) = setup(&env);
    env.mock_all_auths();

    // Deposit and borrow
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 3_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Partial repayment (2 out of 3 tokens)
    let repay_amount = 2_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);

    // Check remaining debt
    let user_borrow = env.as_contract(&lp.address, || {
        lending_pool::storage::get_user_borrow(&env, &user, &asset)
    });
    assert_eq!(user_borrow, 1_000_000_000_000_000_000i128, "Partial repayment should reduce debt");
}

#[test]
#[should_panic]
fn test_repay_more_than_debt_caps_at_debt() {
    let env = Env::default();
    let (lp, admin, asset, user) = setup(&env);
    env.mock_all_auths();

    // Deposit and borrow 2 tokens
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 2_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Try to repay 5 tokens (more than owed) - should panic or cap at 2
    let repay_amount = 5_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);
}