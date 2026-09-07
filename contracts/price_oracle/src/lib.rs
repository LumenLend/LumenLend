#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

pub mod feed;

use feed::ReflectorClient;

/// Storage keys for the PriceOracle contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    ReflectorContract,
    MaxPriceAge,
}

/// Default staleness threshold: 5 minutes (300 seconds).
const DEFAULT_MAX_PRICE_AGE: u64 = 300;

/// Reflector oracle prices are reported in 14-decimal fixed-point.
/// We scale them up to 18-decimal for internal use.
const REFLECTOR_SCALE_FACTOR: i128 = 10_000;

pub trait PriceOracleTrait {
    /// Initialize the oracle with a Reflector contract address.
    /// Uses default max_price_age of 300 seconds.
    fn initialize(env: Env, reflector_contract: Address);

    /// Initialize with a custom staleness threshold.
    fn initialize_with_config(env: Env, reflector_contract: Address, max_price_age: u64);

    /// Returns the USD price of an asset in 18-decimal fixed-point.
    /// Fetches from the Reflector oracle via cross-contract call.
    /// Panics if the price is stale (older than max_price_age).
    fn get_price(env: Env, asset: Address) -> i128;

    /// Returns USD prices for multiple assets in a single call.
    fn get_prices(env: Env, assets: soroban_sdk::Vec<Address>) -> soroban_sdk::Vec<i128>;

    /// Returns the timestamp of the last price update for an asset.
    fn last_updated(env: Env, asset: Address) -> u64;
}

#[contract]
pub struct PriceOracle;

#[contractimpl]
impl PriceOracleTrait for PriceOracle {
    fn initialize(env: Env, reflector_contract: Address) {
        Self::initialize_with_config(env, reflector_contract, DEFAULT_MAX_PRICE_AGE);
    }

    fn initialize_with_config(env: Env, reflector_contract: Address, max_price_age: u64) {
        let storage = env.storage().instance();
        if storage.has(&DataKey::ReflectorContract) {
            panic!("PriceOracle: already initialized");
        }
        storage.set(&DataKey::ReflectorContract, &reflector_contract);
        storage.set(&DataKey::MaxPriceAge, &max_price_age);
    }

    fn get_price(env: Env, asset: Address) -> i128 {
        let result = Self::fetch_price_from_reflector(&env, &asset);
        result.price
    }

    fn get_prices(env: Env, assets: soroban_sdk::Vec<Address>) -> soroban_sdk::Vec<i128> {
        let mut prices = soroban_sdk::Vec::new(&env);
        for asset in assets.iter() {
            let result = Self::fetch_price_from_reflector(&env, &asset);
            prices.push_back(result.price);
        }
        prices
    }

    fn last_updated(env: Env, asset: Address) -> u64 {
        let result = Self::fetch_price_from_reflector(&env, &asset);
        result.timestamp
    }
}

/// Cached result from the oracle fetch including scaled price and raw timestamp.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceResult {
    pub price: i128,
    pub timestamp: u64,
}

impl PriceOracle {
    /// Internal: fetch a price from the Reflector oracle, scale it, and check staleness.
    fn fetch_price_from_reflector(env: &Env, asset: &Address) -> PriceResult {
        let reflector_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReflectorContract)
            .expect("PriceOracle: not initialized");

        let max_price_age: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MaxPriceAge)
            .unwrap_or(DEFAULT_MAX_PRICE_AGE);

        let reflector = ReflectorClient::new(env, &reflector_address);
        let result = reflector.last_price(asset);
        let price_14 = result.price;
        let timestamp = result.timestamp;

        // Check staleness
        let now = env.ledger().timestamp();
        if now > timestamp && (now - timestamp) > max_price_age {
            panic!("PriceOracle: stale price");
        }

        // Scale from 14-decimal to 18-decimal
        let price_18 = price_14
            .checked_mul(REFLECTOR_SCALE_FACTOR)
            .expect("PriceOracle: price overflow during scaling");

        PriceResult {
            price: price_18,
            timestamp,
        }
    }
}
