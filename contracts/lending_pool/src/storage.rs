use soroban_sdk::{contracttype, Address, Env};

/// Per-user position snapshot. Instances of this struct are reconstructed on
/// demand from the per-asset storage keys below (see `get_user_deposit`,
/// `get_user_borrow`, etc.). It is *not* stored verbatim because `Map`
/// values cannot be serialized into instance storage; instead each
/// (user, asset) pair is tracked with its own key.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPosition {
    pub deposits: soroban_sdk::Map<Address, i128>,
    pub borrows: soroban_sdk::Map<Address, i128>,
}

/// Risk parameters configured per supported asset.
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

/// Accrual / liquidity state tracked per supported asset.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetState {
    pub total_deposits: i128,
    pub total_borrows: i128,
    pub last_update_timestamp: u64,
    pub borrow_index: i128,
    pub deposit_index: i128,
}

/// All storage keys for the LendingPool contract. Keys are stored in
/// instance storage.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    Admin,
    Oracle,
    InterestRateModel,
    AssetConfig(Address),
    AssetState(Address),
    UserDeposit(Address, Address),
    UserBorrow(Address, Address),
}

// ---------------------------------------------------------------------------
// AssetConfig helpers
// ---------------------------------------------------------------------------

pub fn read_asset_config(env: &Env, asset: &Address) -> Option<AssetConfig> {
    env.storage()
        .instance()
        .get(&DataKey::AssetConfig(asset.clone()))
}

pub fn write_asset_config(env: &Env, asset: &Address, config: &AssetConfig) {
    env.storage()
        .instance()
        .set(&DataKey::AssetConfig(asset.clone()), config);
}

pub fn has_asset_config(env: &Env, asset: &Address) -> bool {
    env.storage()
        .instance()
        .has(&DataKey::AssetConfig(asset.clone()))
}

// ---------------------------------------------------------------------------
// AssetState helpers
// ---------------------------------------------------------------------------

pub fn read_asset_state(env: &Env, asset: &Address) -> Option<AssetState> {
    env.storage()
        .instance()
        .get(&DataKey::AssetState(asset.clone()))
}

pub fn write_asset_state(env: &Env, asset: &Address, state: &AssetState) {
    env.storage()
        .instance()
        .set(&DataKey::AssetState(asset.clone()), state);
}

// ---------------------------------------------------------------------------
// UserPosition helpers (per-asset storage keys)
// ---------------------------------------------------------------------------

pub fn get_user_deposit(env: &Env, user: &Address, asset: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::UserDeposit(user.clone(), asset.clone()))
        .unwrap_or(0)
}

pub fn set_user_deposit(env: &Env, user: &Address, asset: &Address, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::UserDeposit(user.clone(), asset.clone()), &amount);
}

pub fn get_user_borrow(env: &Env, user: &Address, asset: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::UserBorrow(user.clone(), asset.clone()))
        .unwrap_or(0)
}

pub fn set_user_borrow(env: &Env, user: &Address, asset: &Address, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::UserBorrow(user.clone(), asset.clone()), &amount);
}

// ---------------------------------------------------------------------------
// Admin / oracle / interest rate model helpers
// ---------------------------------------------------------------------------

pub fn read_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn read_oracle(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Oracle)
}

pub fn write_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&DataKey::Oracle, oracle);
}

pub fn read_interest_rate_model(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::InterestRateModel)
}

pub fn write_interest_rate_model(env: &Env, irm: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::InterestRateModel, irm);
}
