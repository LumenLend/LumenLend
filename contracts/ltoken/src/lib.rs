#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct LToken;

#[contractimpl]
impl LToken {
    /// Placeholder scaffold — full implementation lands in later days.
    pub fn version(env: Env) -> u32 {
        env.ledger().sequence()
    }
}
