use soroban_sdk::{Address, Env};

use crate::DataKey;

pub fn mint(env: Env, to: Address, amount: i128) {
    let storage = env.storage().instance();
    let lending_pool: Address = storage
        .get(&DataKey::LendingPool)
        .expect("LToken: not initialized");
    lending_pool.require_auth();

    let total_supply: i128 = storage.get(&DataKey::TotalSupply).unwrap_or(0);
    let new_total_supply = total_supply
        .checked_add(amount)
        .expect("LToken: total supply overflow");
    storage.set(&DataKey::TotalSupply, &new_total_supply);

    let balance: i128 = storage.get(&DataKey::Balance(to.clone())).unwrap_or(0);
    let new_balance = balance
        .checked_add(amount)
        .expect("LToken: balance overflow");
    storage.set(&DataKey::Balance(to), &new_balance);
}
