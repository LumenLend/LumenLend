use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

/// A generic client for interacting with SEP-41 or compatible token contracts.
///
/// This wraps `env.invoke_contract` so lending_pool can transfer tokens,
/// approve allowances, and read balances of arbitrary asset contracts.
pub struct TokenClient {
    env: Env,
    contract_id: Address,
}

impl TokenClient {
    pub fn new(env: &Env, contract_id: &Address) -> Self {
        Self {
            env: env.clone(),
            contract_id: contract_id.clone(),
        }
    }

    /// Call the underlying token's `transfer` function (SEP-41).
    /// Signature: `transfer(from, to, amount)`.
    pub fn transfer(&self, from: &Address, to: &Address, amount: &i128) {
        let args: Vec<Val> = (from.clone(), to.clone(), *amount).into_val(&self.env);
        self.env.invoke_contract::<Val>(
            &self.contract_id,
            &Symbol::new(&self.env, "transfer"),
            args,
        );
    }

    /// Call the underlying token's `transfer_from` function.
    pub fn transfer_from(&self, spender: &Address, from: &Address, to: &Address, amount: &i128) {
        let args: Vec<Val> = (spender.clone(), from.clone(), to.clone(), *amount).into_val(&self.env);
        self.env.invoke_contract::<Val>(
            &self.contract_id,
            &Symbol::new(&self.env, "transfer_from"),
            args,
        );
    }

    /// Call the underlying token's `approve` function.
    pub fn approve(&self, spender: &Address, amount: &i128, expiration_ledger: u32) {
        let args: Vec<Val> = (spender.clone(), *amount, expiration_ledger).into_val(&self.env);
        self.env.invoke_contract::<Val>(
            &self.contract_id,
            &Symbol::new(&self.env, "approve"),
            args,
        );
    }

    /// Call the underlying token's `balance` function.
    pub fn balance(&self, owner: &Address) -> i128 {
        let args: Vec<Val> = (owner.clone(),).into_val(&self.env);
        self.env.invoke_contract::<i128>(
            &self.contract_id,
            &Symbol::new(&self.env, "balance"),
            args,
        )
    }
}