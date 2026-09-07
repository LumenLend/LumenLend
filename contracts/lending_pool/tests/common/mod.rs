use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Storage keys for the MockOracle contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OracleDataKey {
    /// Per-asset price override in 18-decimal fixed-point.
    Price(Address),
}

#[contract]
pub struct MockOracle;

pub trait MockOracleTrait {
    fn set_price(env: Env, asset: Address, price: i128);
    fn get_price(env: Env, asset: Address) -> i128;
}

#[contractimpl]
impl MockOracleTrait for MockOracle {
    fn set_price(env: Env, asset: Address, price: i128) {
        env.storage().instance().set(&OracleDataKey::Price(asset), &price);
    }

    fn get_price(env: Env, asset: Address) -> i128 {
        env.storage()
            .instance()
            .get(&OracleDataKey::Price(asset))
            .unwrap_or(1_000_000_000_000_000_000i128)
    }
}

/// Storage keys for the MockToken contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenDataKey {
    Balance(Address),
    TotalSupply,
}

/// A minimal SEP-41-compatible token used as a test double for the underlying
/// asset. Supports mint, transfer, and transfer_from so the LendingPool can
/// move funds during deposit/borrow/repay flows.
#[contract]
pub struct MockToken;

pub trait MockTokenTrait {
    fn mint(env: Env, to: Address, amount: i128);
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
    fn transfer_from(env: Env, _spender: Address, from: Address, to: Address, amount: i128);
    fn balance(env: Env, owner: Address) -> i128;
    fn total_supply(env: Env) -> i128;
}

#[contractimpl]
impl MockTokenTrait for MockToken {
    fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
        let balance = env
            .storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TokenDataKey::Balance(to), &(balance + amount));
        let supply = env.storage().instance().get::<_, i128>(&TokenDataKey::TotalSupply).unwrap_or(0);
        env.storage().instance().set(&TokenDataKey::TotalSupply, &(supply + amount));
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let from_balance = env
            .storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_balance >= amount, "MockToken: insufficient balance");
        let to_balance = env
            .storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage().instance().set(&TokenDataKey::Balance(from), &(from_balance - amount));
        env.storage().instance().set(&TokenDataKey::Balance(to), &(to_balance + amount));
    }

    fn transfer_from(env: Env, _spender: Address, from: Address, to: Address, amount: i128) {
        let from_balance = env
            .storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_balance >= amount, "MockToken: insufficient balance");
        let to_balance = env
            .storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage().instance().set(&TokenDataKey::Balance(from), &(from_balance - amount));
        env.storage().instance().set(&TokenDataKey::Balance(to), &(to_balance + amount));
    }

    fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::Balance(owner))
            .unwrap_or(0)
    }

    fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get::<_, i128>(&TokenDataKey::TotalSupply)
            .unwrap_or(0)
    }
}

/// Registers an underlying token at `asset`, a real LToken contract at
/// `ltoken`, and returns `(token_client, ltoken_id)`.
pub fn deploy_market(env: &Env, pool_id: &Address, asset: &Address) -> Address {
    env.register_contract(Some(asset), MockToken);

    // Deploy a real LToken contract for this asset.
    let ltoken_id = env.register_contract(
        None,
        ltoken::LToken,
    );
    let ltoken_client = ltoken::LTokenClient::new(env, &ltoken_id);
    ltoken_client.initialize(
        pool_id,
        asset,
        &soroban_sdk::String::from_str(env, "Lumen XLM"),
        &soroban_sdk::String::from_str(env, "lXLM"),
    );

    ltoken_id
}