//! Storage keys and typed accessors for the AnchorNet contract.
//!
//! All persistent entries use the `persistent` storage with a TTL that is
//! extended on every read/write so that active pools are not archived.

use soroban_sdk::{contracttype, Address, Env, Symbol};

use crate::types::{Pool, Settlement};

const DAY_IN_LEDGERS: u32 = 17_280;
/// How long an entry's TTL is extended to on access (~30 days).
const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Extend the TTL once it drops below this threshold (~29 days).
const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Keys for every entry the contract stores.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The contract administrator.
    Admin,
    /// Whether an address is a registered anchor.
    Anchor(Address),
    /// The aggregate [`Pool`] for an asset.
    Pool(Symbol),
    /// A provider's liquidity balance in a given asset.
    Balance(Address, Symbol),
    /// Whether the contract is paused.
    Paused,
    /// The protocol fee in basis points.
    FeeBps,
    /// Monotonic counter for settlement ids.
    SettlementCount,
    /// A settlement record by id.
    Settlement(u64),
    /// Protocol fees accrued (and not yet collected) for an asset.
    FeesAccrued(Symbol),
}

fn extend(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

/// Returns `true` once an administrator has been set.
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Reads the administrator address. Panics if uninitialized — callers should
/// guard with [`has_admin`] first.
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

/// Persists the administrator address in instance storage.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Returns `true` if the contract is currently paused.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

/// Sets the paused flag.
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

/// Reads the protocol fee in basis points (defaults to zero if unset).
pub fn get_fee_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0)
}

/// Persists the protocol fee in basis points.
pub fn set_fee_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::FeeBps, &bps);
}

/// Returns `true` if `anchor` has been registered.
pub fn is_anchor(env: &Env, anchor: &Address) -> bool {
    let key = DataKey::Anchor(anchor.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

/// Marks `anchor` as registered.
pub fn set_anchor(env: &Env, anchor: &Address) {
    set_anchor_flag(env, anchor, true);
}

/// Sets the registration flag for `anchor`.
pub fn set_anchor_flag(env: &Env, anchor: &Address, registered: bool) {
    let key = DataKey::Anchor(anchor.clone());
    env.storage().persistent().set(&key, &registered);
    extend(env, &key);
}

/// Reads the [`Pool`] for `asset`, returning an empty pool if none exists.
pub fn get_pool(env: &Env, asset: &Symbol) -> Pool {
    let key = DataKey::Pool(asset.clone());
    match env.storage().persistent().get::<DataKey, Pool>(&key) {
        Some(pool) => {
            extend(env, &key);
            pool
        }
        None => Pool::empty(asset.clone()),
    }
}

/// Returns `true` if a pool entry exists for `asset`.
pub fn has_pool(env: &Env, asset: &Symbol) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Pool(asset.clone()))
}

/// Persists `pool` for `asset`.
pub fn set_pool(env: &Env, asset: &Symbol, pool: &Pool) {
    let key = DataKey::Pool(asset.clone());
    env.storage().persistent().set(&key, pool);
    extend(env, &key);
}

/// Reads a provider's balance in `asset` (zero if none).
pub fn get_balance(env: &Env, provider: &Address, asset: &Symbol) -> i128 {
    let key = DataKey::Balance(provider.clone(), asset.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Persists a provider's balance in `asset`.
pub fn set_balance(env: &Env, provider: &Address, asset: &Symbol, amount: i128) {
    let key = DataKey::Balance(provider.clone(), asset.clone());
    env.storage().persistent().set(&key, &amount);
    extend(env, &key);
}

/// Reads the settlement id counter (zero before the first settlement).
pub fn get_settlement_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::SettlementCount)
        .unwrap_or(0)
}

/// Persists the settlement id counter.
pub fn set_settlement_count(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::SettlementCount, &count);
}

/// Reads a settlement by id, if it exists.
pub fn get_settlement(env: &Env, id: u64) -> Option<Settlement> {
    let key = DataKey::Settlement(id);
    let found = env.storage().persistent().get(&key);
    if found.is_some() {
        extend(env, &key);
    }
    found
}

/// Persists a settlement record.
pub fn set_settlement(env: &Env, settlement: &Settlement) {
    let key = DataKey::Settlement(settlement.id);
    env.storage().persistent().set(&key, settlement);
    extend(env, &key);
}

/// Reads the accrued (uncollected) protocol fees for `asset`.
pub fn get_fees_accrued(env: &Env, asset: &Symbol) -> i128 {
    let key = DataKey::FeesAccrued(asset.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

/// Persists the accrued protocol fees for `asset`.
pub fn set_fees_accrued(env: &Env, asset: &Symbol, amount: i128) {
    let key = DataKey::FeesAccrued(asset.clone());
    env.storage().persistent().set(&key, &amount);
    extend(env, &key);
}
