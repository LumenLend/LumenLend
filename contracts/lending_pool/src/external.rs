use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

/// Minimal cross-contract client for the LToken contract.
/// Keeps the lending pool from linking the full `ltoken` contract, which
/// would emit duplicate contract entrypoint exports in the wasm build.
pub struct LTokenClient(Env, Address);

impl LTokenClient {
    pub fn new(env: &Env, contract_id: &Address) -> Self {
        Self(env.clone(), contract_id.clone())
    }

    pub fn mint(&self, to: &Address, amount: &i128) {
        let args: Vec<Val> = (to.clone(), *amount).into_val(&self.0);
        self.0.invoke_contract::<()>(
            &self.1,
            &Symbol::new(&self.0, "mint"),
            args,
        );
    }

    pub fn burn(&self, from: &Address, amount: &i128) {
        let args: Vec<Val> = (from.clone(), *amount).into_val(&self.0);
        self.0.invoke_contract::<()>(
            &self.1,
            &Symbol::new(&self.0, "burn"),
            args,
        );
    }

    pub fn exchange_rate(&self) -> i128 {
        let args: Vec<Val> = Vec::new(&self.0);
        self.0.invoke_contract::<i128>(
            &self.1,
            &Symbol::new(&self.0, "exchange_rate"),
            args,
        )
    }
}

/// Minimal cross-contract client for the InterestRateModel contract.
/// Mirrors the `InterestRateModelClient` without linking the contract crate.
pub struct InterestRateModelClient(Env, Address);

impl InterestRateModelClient {
    pub fn new(env: &Env, contract_id: &Address) -> Self {
        Self(env.clone(), contract_id.clone())
    }

    pub fn get_borrow_rate(&self, total_deposits: &i128, total_borrows: &i128) -> i128 {
        let args: Vec<Val> = (*total_deposits, *total_borrows).into_val(&self.0);
        self.0.invoke_contract::<i128>(
            &self.1,
            &Symbol::new(&self.0, "get_borrow_rate"),
            args,
        )
    }
}