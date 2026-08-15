use ltoken::{LToken, LTokenClient};
use soroban_sdk::{
    testutils::Address as TestAddress,
    Address, Env, String,
};

const ONE: i128 = 1_000_000_000_000_000_000;

fn generate_address(env: &Env, _id: &str) -> Address {
    Address::generate(env)
}

fn with_ltoken<F: FnOnce(&Env, &LTokenClient)>(f: F) {
    let env = Env::default();
    let contract_id = env.register_contract(None, LToken);
    let client = LTokenClient::new(&env, &contract_id);
    let lending_pool = generate_address(&env, "lending_pool");
    let underlying = generate_address(&env, "underlying");
    client.initialize(&lending_pool, &underlying, &String::from_str(&env, "Test"), &String::from_str(&env, "TST"));
    f(&env, &client);
}

#[test]
fn test_initialize_sets_fields() {
    with_ltoken(|env, client| {
        assert_eq!(client.total_supply(), 0);
        assert_eq!(client.exchange_rate(), ONE);
        let unknown = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
        assert_eq!(client.balance(&unknown), 0);
    });
}

#[test]
#[should_panic]
fn test_mint_unauthorized_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LToken);
    let client = LTokenClient::new(&env, &contract_id);
    let lending_pool = generate_address(&env, "GD5GCVMRXCLGELRFJECHWZ46Y3LIIFSJ3SLBWWY5MP33MOXSDTISRUKA");
    let underlying = generate_address(&env, "GASOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GAAAA");
    client.initialize(&lending_pool, &underlying, &String::from_str(&env, "Test"), &String::from_str(&env, "TST"));
    let to = generate_address(&env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
    client.mint(&to, &100);
}

#[test]
fn test_mint_increases_supply_and_balance() {
    with_ltoken(|env, client| {
        env.mock_all_auths();
        let to = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
        client.mint(&to, &100);
        assert_eq!(client.total_supply(), 100);
        assert_eq!(client.balance(&to), 100);
    });
}

#[test]
#[should_panic]
fn test_burn_unauthorized_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LToken);
    let client = LTokenClient::new(&env, &contract_id);
    let lending_pool = generate_address(&env, "GD5GCVMRXCLGELRFJECHWZ46Y3LIIFSJ3SLBWWY5MP33MOXSDTISRUKA");
    let underlying = generate_address(&env, "GASOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GAAAA");
    client.initialize(&lending_pool, &underlying, &String::from_str(&env, "Test"), &String::from_str(&env, "TST"));
    let from = generate_address(&env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
    client.burn(&from, &100);
}

#[test]
fn test_burn_decreases_supply_and_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LToken);
    let client = LTokenClient::new(&env, &contract_id);
    let lending_pool = generate_address(&env, "GD5GCVMRXCLGELRFJECHWZ46Y3LIIFSJ3SLBWWY5MP33MOXSDTISRUKA");
    let underlying = generate_address(&env, "GASOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GAAAA");
    client.initialize(&lending_pool, &underlying, &String::from_str(&env, "Test"), &String::from_str(&env, "TST"));
    env.mock_all_auths();
    let from = generate_address(&env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
    client.mint(&from, &100);
    client.burn(&from, &50);
    assert_eq!(client.total_supply(), 50);
    assert_eq!(client.balance(&from), 50);
}

#[test]
fn test_balance_returns_zero_for_unknown_address() {
    with_ltoken(|env, client| {
        let unknown = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
        assert_eq!(client.balance(&unknown), 0);
    });
}

#[test]
fn test_exchange_rate_starts_at_one() {
    with_ltoken(|_env, client| {
        assert_eq!(client.exchange_rate(), ONE);
    });
}

#[test]
fn test_exchange_rate_zero_supply_does_not_panic() {
    with_ltoken(|_env, client| {
        assert_eq!(client.total_supply(), 0);
        assert_eq!(client.exchange_rate(), ONE);
    });
}

#[test]
fn test_transfer() {
    with_ltoken(|env, client| {
        env.mock_all_auths();
        let from = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
        let to = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GCCCC");
        client.mint(&from, &100);
        client.transfer(&from, &to, &50);
        assert_eq!(client.balance(&from), 50);
        assert_eq!(client.balance(&to), 50);
    });
}

#[test]
fn test_transfer_from() {
    with_ltoken(|env, client| {
        env.mock_all_auths();
        let owner = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GBBBB");
        let spender = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GCCCC");
        let to = generate_address(env, "GBCUOZBYEINXGJQHDOPNEOOLXD6D6RLJOMZ3DX3Z4TX66N4I6T6GDDDD");
        client.mint(&owner, &100);
        client.approve(&owner, &spender, &50);
        client.transfer_from(&spender, &owner, &to, &50);
        assert_eq!(client.balance(&owner), 50);
        assert_eq!(client.balance(&to), 50);
        assert_eq!(client.allowance(&owner, &spender), 0);
    });
}
