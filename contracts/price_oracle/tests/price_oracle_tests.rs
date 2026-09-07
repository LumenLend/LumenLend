use price_oracle::{PriceOracle, PriceOracleClient, feed::ReflectorPriceResponse};
use soroban_sdk::{testutils::Address as TestAddress, Address, contracttype, Env, testutils::Ledger};

/// A mock Reflector oracle that emulates `lastprice`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
enum MockReflectorKey {
    Price,
    Timestamp,
}

#[soroban_sdk::contract]
pub struct MockReflector;

#[soroban_sdk::contractimpl]
impl MockReflector {
    pub fn set_last_price(env: Env, price: i128, timestamp: u64) {
        env.storage().instance().set(&MockReflectorKey::Price, &price);
        env.storage().instance().set(&MockReflectorKey::Timestamp, &timestamp);
    }

    pub fn lastprice(env: Env, _asset: Address) -> ReflectorPriceResponse {
        let price = env.storage().instance().get::<_, i128>(&MockReflectorKey::Price).unwrap_or(0);
        let timestamp = env.storage().instance().get::<_, u64>(&MockReflectorKey::Timestamp).unwrap_or(0);
        ReflectorPriceResponse { price, timestamp }
    }
}

fn setup(env: &Env) -> (PriceOracleClient, Address) {
    let reflector = env.register_contract(None, MockReflector);
    let oracle = PriceOracleClient::new(env, &env.register_contract(None, PriceOracle));
    oracle.initialize(&reflector);
    (oracle, reflector)
}

#[test]
#[should_panic]
fn test_double_initialize_panics() {
    let env = Env::default();
    let reflector = env.register_contract(None, MockReflector);
    let oracle = PriceOracleClient::new(&env, &env.register_contract(None, PriceOracle));
    oracle.initialize(&reflector);
    oracle.initialize(&reflector);
}

#[test]
#[should_panic]
fn test_stale_price_panics() {
    let env = Env::default();
    let (oracle, reflector) = setup(&env);

    let mock = MockReflectorClient::new(&env, &reflector);
    let asset = Address::generate(&env);
    // Report a price with an old timestamp, then advance the ledger well past
    // the staleness window (default max_price_age = 300s).
    mock.set_last_price(&1_000_000_000_000_00i128, &1_000);
    env.ledger().set_timestamp(1_000 + 301);

    let _ = oracle.get_price(&asset);
}

#[test]
fn test_get_price_scales_14_to_18_decimal() {
    let env = Env::default();
    let (oracle, reflector) = setup(&env);

    let mock = MockReflectorClient::new(&env, &reflector);
    let asset = Address::generate(&env);
    // Reflector reports price in 14-decimal fixed point. 1.0 USD = 1e14.
    mock.set_last_price(&1_000_000_000_000_00i128, &env.ledger().timestamp());

    let price = oracle.get_price(&asset);
    // Scale by 1e4: 1e14 * 1e4 = 1e18.
    assert_eq!(price, 1_000_000_000_000_000_000i128);
}

#[test]
fn test_get_prices_batch() {
    let env = Env::default();
    let (oracle, reflector) = setup(&env);

    let mock = MockReflectorClient::new(&env, &reflector);
    mock.set_last_price(&2_000_000_000_000_00i128, &env.ledger().timestamp());

    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let mut assets = soroban_sdk::Vec::new(&env);
    assets.push_back(a1);
    assets.push_back(a2);

    let prices = oracle.get_prices(&assets);
    assert_eq!(prices.len(), 2);
    assert_eq!(prices.get(0).unwrap(), 2_000_000_000_000_000_000i128);
    assert_eq!(prices.get(1).unwrap(), 2_000_000_000_000_000_000i128);
}