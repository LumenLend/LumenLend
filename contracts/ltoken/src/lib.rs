#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

mod mint;
mod burn;

#[contract]
pub struct LToken;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    LendingPool,
    UnderlyingAsset,
    Name,
    Symbol,
    TotalSupply,
    ExchangeRate,
    TotalUnderlyingDeposited,
    Balance(Address),
    Allowance(Address, Address),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPosition {
    pub deposits: soroban_sdk::Map<Address, i128>,
    pub borrows: soroban_sdk::Map<Address, i128>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetConfig {
    pub ltv: u32,
    pub liquidation_threshold: u32,
    pub liquidation_bonus: u32,
    pub reserve_factor: u32,
    pub ltoken_address: Address,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetState {
    pub total_deposits: i128,
    pub total_borrows: i128,
    pub last_update_timestamp: u64,
    pub borrow_index: i128,
    pub deposit_index: i128,
}

pub trait LTokenTrait {
    fn initialize(env: Env, lending_pool: Address, underlying_asset: Address, name: soroban_sdk::String, symbol: soroban_sdk::String);
    fn mint(env: Env, to: Address, amount: i128);
    fn burn(env: Env, from: Address, amount: i128);
    fn balance(env: Env, owner: Address) -> i128;
    fn total_supply(env: Env) -> i128;
    fn underlying_asset(env: Env) -> Address;
    fn exchange_rate(env: Env) -> i128;
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128);
    fn approve(env: Env, owner: Address, spender: Address, amount: i128);
    fn allowance(env: Env, owner: Address, spender: Address) -> i128;
}

#[contractimpl]
impl LTokenTrait for LToken {
    fn initialize(env: Env, lending_pool: Address, underlying_asset: Address, name: soroban_sdk::String, symbol: soroban_sdk::String) {
        let storage = env.storage().instance();
        let existing: Option<Address> = storage.get(&DataKey::LendingPool);
        if existing.is_some() {
            panic!("LToken: already initialized");
        }
        storage.set(&DataKey::LendingPool, &lending_pool);
        storage.set(&DataKey::UnderlyingAsset, &underlying_asset);
        storage.set(&DataKey::Name, &name);
        storage.set(&DataKey::Symbol, &symbol);
        storage.set(&DataKey::TotalSupply, &0i128);
        storage.set(&DataKey::TotalUnderlyingDeposited, &0i128);
        storage.set(&DataKey::ExchangeRate, &1_000_000_000_000_000_000i128);
    }

    fn mint(env: Env, to: Address, amount: i128) {
        mint::mint(env, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        burn::burn(env, from, amount)
    }

    fn balance(env: Env, owner: Address) -> i128 {
        let storage = env.storage().instance();
        storage.get(&DataKey::Balance(owner)).unwrap_or(0)
    }

    fn total_supply(env: Env) -> i128 {
        let storage = env.storage().instance();
        storage.get(&DataKey::TotalSupply).unwrap_or(0)
    }

    fn underlying_asset(env: Env) -> Address {
        let storage = env.storage().instance();
        let underlying: Address = storage
            .get(&DataKey::UnderlyingAsset)
            .expect("LToken: not initialized");
        underlying
    }

    fn exchange_rate(env: Env) -> i128 {
        let storage = env.storage().instance();
        let total_supply: i128 = storage.get(&DataKey::TotalSupply).unwrap_or(0);
        if total_supply == 0 {
            return 1_000_000_000_000_000_000i128;
        }
        let total_underlying: i128 = storage.get(&DataKey::TotalUnderlyingDeposited).unwrap_or(0);
        total_underlying
            .checked_mul(1_000_000_000_000_000_000i128)
            .expect("LToken: overflow")
            .checked_div(total_supply)
            .expect("LToken: division by zero")
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let storage = env.storage().instance();
        let balance: i128 = storage.get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let new_balance = balance
            .checked_sub(amount)
            .expect("LToken: insufficient balance");
        storage.set(&DataKey::Balance(from.clone()), &new_balance);
        let to_balance: i128 = storage.get(&DataKey::Balance(to.clone())).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(amount)
            .expect("LToken: balance overflow");
        storage.set(&DataKey::Balance(to), &new_to_balance);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        let storage = env.storage().instance();
        let allowance: i128 = storage
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        let new_allowance = allowance
            .checked_sub(amount)
            .expect("LToken: insufficient allowance");
        storage.set(&DataKey::Allowance(from.clone(), spender.clone()), &new_allowance);

        let balance: i128 = storage.get(&DataKey::Balance(from.clone())).unwrap_or(0);
        let new_balance = balance
            .checked_sub(amount)
            .expect("LToken: insufficient balance");
        storage.set(&DataKey::Balance(from.clone()), &new_balance);
        let to_balance: i128 = storage.get(&DataKey::Balance(to.clone())).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(amount)
            .expect("LToken: balance overflow");
        storage.set(&DataKey::Balance(to), &new_to_balance);
    }

    fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        let storage = env.storage().instance();
        storage.set(&DataKey::Allowance(owner, spender), &amount);
    }

    fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        let storage = env.storage().instance();
        storage.get(&DataKey::Allowance(owner, spender)).unwrap_or(0)
    }
}
