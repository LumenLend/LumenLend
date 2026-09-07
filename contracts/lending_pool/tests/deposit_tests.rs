mod common;

use soroban_sdk::{testutils::Address as TestAddress, Address, Env};
use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};

use common::{deploy_market, MockOracle, MockTokenClient};

fn setup(env: &Env) -> (LendingPoolClient<'_>, Address, Address, Address) {
    let admin = Address::generate(env);
    let user = Address::generate(env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(env, &irm_id);
    let base: u32 = 200;
    let slope1: u32 = 1000;
    let slope2: u32 = 30000;
    let optimal: u32 = 8000;
    irm.initialize(&base, &slope1, &slope2, &optimal);

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
#[should_panic]
fn test_deposit_zero_amount_fails() {
    let env = Env::default();
    let (lp, _admin, asset, user) = setup(&env);
    env.mock_all_auths();
    lp.deposit(&user, &asset, &0);
}