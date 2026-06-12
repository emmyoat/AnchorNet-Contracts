//! Storage keys and typed accessors for the AnchorNet contract.
//!
//! All persistent entries use the `persistent` storage with a TTL that is
//! extended on every read/write so that active pools are not archived.

use soroban_sdk::{contracttype, Address, Env, Symbol};

use crate::types::Pool;

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

/// Returns `true` if `anchor` has been registered.
pub fn is_anchor(env: &Env, anchor: &Address) -> bool {
    let key = DataKey::Anchor(anchor.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

/// Marks `anchor` as registered.
pub fn set_anchor(env: &Env, anchor: &Address) {
    let key = DataKey::Anchor(anchor.clone());
    env.storage().persistent().set(&key, &true);
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
