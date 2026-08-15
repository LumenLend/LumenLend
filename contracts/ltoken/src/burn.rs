use soroban_sdk::{Address, Env};

use crate::DataKey;

pub fn burn(env: Env, from: Address, amount: i128) {
    let storage = env.storage().instance();
    let lending_pool: Address = storage
        .get(&DataKey::LendingPool)
        .expect("LToken: not initialized");
    lending_pool.require_auth();

    let balance: i128 = storage.get(&DataKey::Balance(from.clone())).unwrap_or(0);
    let new_balance = balance
        .checked_sub(amount)
        .expect("LToken: insufficient balance");
    storage.set(&DataKey::Balance(from.clone()), &new_balance);

    let total_supply: i128 = storage.get(&DataKey::TotalSupply).unwrap_or(0);
    let new_total_supply = total_supply
        .checked_sub(amount)
        .expect("LToken: total supply underflow");
    storage.set(&DataKey::TotalSupply, &new_total_supply);
}
