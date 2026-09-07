// Deposit tests for LendingPool

use soroban_sdk::{testutils::Address as TestAddress, Env};
use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};

fn irm_client(env: &Env) -> InterestRateModelClient {
    let irm_id = env.register_contract(None, InterestRateModel);
    InterestRateModelClient::new(&env, &irm_id)
}

fn irm_init(irm: &InterestRateModelClient, env: &Env) {
    irm.initialize(
        env.clone(),
        200,     // base_rate = 2%
        1000,    // slope1 = 10% at optimal
        30000,   // slope2 = 300% at 100%
        8000,    // optimal_utilization = 80%
    );
}

fn supported_asset(env: &Env) -> (TestAddress, u32, u32, u32, u32) {
    let asset = TestAddress::generate(env);
    (asset, 7500, 8000, 500, 1000)
}

fn lending_pool_client(env: &Env, irm: &InterestRateModelClient) -> LendingPoolClient {
    let lp_id = env.register_contract(None, LendingPool);
    LendingPoolClient::new(&env, &lp_id)
}

#[test]
#[should_panic]
fn test_deposit_zero_amount_fails() {
    let env = Env::default();
    let admin = TestAddress::generate(&env);
    let oracle = TestAddress::generate(&env);
    let user = TestAddress::generate(&env);

    let irm = irm_client(&env);
    irm_init(&irm, &env);

    let (asset, _ltv, _lt, _lb, _rf) = supported_asset(&env);
    let lp = lending_pool_client(&env, &irm);

    let amount = 0i128;
    lp.deposit(env.clone(), user.clone(), asset.clone(), amount); // Should panic
}