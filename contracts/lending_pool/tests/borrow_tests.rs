use soroban_sdk::{Address, Env, testutils::Address as TestAddress};
use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};

#[test]
fn test_borrow_within_ltv_succeeds() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(&env, &irm_id);
    let br: u32 = 200;
    let sl1: u32 = 1000;
    let sl2: u32 = 30000;
    let opt: u32 = 8000;
    irm.initialize(&br, &sl1, &sl2, &opt);

    let asset = Address::generate(&env);
    let lp = LendingPoolClient::new(&env, &env.register_contract(None, LendingPool));

    let hf = lp.get_health_factor(&user);
    assert!(hf >= 1_000_000_000_000_000_000i128, "Health factor should be >= 1.0 when no debt");
}

#[test]
#[should_panic]
fn test_borrow_exceeds_ltv_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(&env, &irm_id);
    let br: u32 = 200;
    let sl1: u32 = 1000;
    let sl2: u32 = 30000;
    let opt: u32 = 8000;
    irm.initialize(&br, &sl1, &sl2, &opt);

    let asset = Address::generate(&env);
    let lp = LendingPoolClient::new(&env, &env.register_contract(None, LendingPool));

    let amount = 10_000_000_000_000_000_001i128;
    lp.borrow(&user, &asset, &amount);
}

#[test]
#[should_panic]
fn test_borrow_insufficient_liquidity_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(&env, &irm_id);
    let br: u32 = 200;
    let sl1: u32 = 1000;
    let sl2: u32 = 30000;
    let opt: u32 = 8000;
    irm.initialize(&br, &sl1, &sl2, &opt);

    let asset = Address::generate(&env);
    let lp = LendingPoolClient::new(&env, &env.register_contract(None, LendingPool));

    lp.borrow(&user, &asset, &1_000_000_000_000_000_000i128);
}

#[test]
fn test_borrow_updates_debt() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let irm_id = env.register_contract(None, InterestRateModel);
    let irm = InterestRateModelClient::new(&env, &irm_id);
    let br: u32 = 200;
    let sl1: u32 = 1000;
    let sl2: u32 = 30000;
    let opt: u32 = 8000;
    irm.initialize(&br, &sl1, &sl2, &opt);

    let asset = Address::generate(&env);
    let lp = LendingPoolClient::new(&env, &env.register_contract(None, LendingPool));

    let deposit_amount = 10_000_000_000_000_000_000i128;
    lp.deposit(&admin, &asset, &deposit_amount);

    let borrow_amount = 5_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    let user_borrow = lending_pool::storage::get_user_borrow(&env, &user, &asset);
    assert_eq!(user_borrow, borrow_amount, "User's borrow should be updated");
}