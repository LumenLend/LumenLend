use soroban_sdk::{Address, contracttype, Env, IntoVal, Symbol, Vec};

/// A price response from the Reflector oracle.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectorPriceResponse {
    pub price: i128,
    pub timestamp: u64,
}

/// Client for interacting with a Reflector oracle contract.
///
/// Issues a `lastprice(asset)` cross-contract call.
pub struct ReflectorClient {
    env: Env,
    contract_id: Address,
}

impl ReflectorClient {
    pub fn new(env: &Env, contract_id: &Address) -> Self {
        Self {
            env: env.clone(),
            contract_id: contract_id.clone(),
        }
    }

    /// Calls `lastprice(asset)` on the Reflector contract.
    /// Returns the price and timestamp of the most recent price update.
    pub fn last_price(&self, asset: &Address) -> ReflectorPriceResponse {
        let args: Vec<soroban_sdk::Val> = (asset.clone(),).into_val(&self.env);
        self.env.invoke_contract::<ReflectorPriceResponse>(
            &self.contract_id,
            &Symbol::new(&self.env, "lastprice"),
            args,
        )
    }
}
