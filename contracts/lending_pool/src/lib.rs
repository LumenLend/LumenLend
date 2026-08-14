#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct LendingPool;

#[contractimpl]
impl LendingPool {
    /// Placeholder scaffold — full implementation lands in later days.
    pub fn version(env: Env) -> u32 {
        env.ledger().sequence()
    }
}
