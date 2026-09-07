use soroban_sdk::{Address, Env, testutils::Address as TestAddress};
use lending_pool::{LendingPool, LendingPoolClient};
use interest_rate_model::{InterestRateModel, InterestRateModelClient};

#[test]
fn test_full_repayment_clears_debt() {
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

    // Deposit and borrow to set up debt
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 3_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Full repayment
    let repay_amount = 3_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);

    // Check debt is cleared
    let user_borrow = lending_pool::storage::get_user_borrow(&env, &user, &asset);
    assert_eq!(user_borrow, 0, "Full repayment should clear all debt");
}

#[test]
fn test_partial_repayment_reduces_debt() {
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

    // Deposit and borrow
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 3_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Partial repayment (2 out of 3 tokens)
    let repay_amount = 2_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);

    // Check remaining debt
    let user_borrow = lending_pool::storage::get_user_borrow(&env, &user, &asset);
    assert_eq!(user_borrow, 1_000_000_000_000_000_000i128, "Partial repayment should reduce debt");
}

#[test]
#[should_panic]
fn test_repay_more_than_debt_caps_at_debt() {
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

    // Deposit and borrow 2 tokens
    lp.deposit(&admin, &asset, &10_000_000_000_000_000_000i128);

    let borrow_amount = 2_000_000_000_000_000_000i128;
    lp.borrow(&user, &asset, &borrow_amount);

    // Try to repay 5 tokens (more than owed) - should panic or cap at 2
    let repay_amount = 5_000_000_000_000_000_000i128;
    lp.repay(&user, &asset, &repay_amount);
}