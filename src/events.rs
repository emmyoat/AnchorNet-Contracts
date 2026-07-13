//! Event publishing helpers.
//!
//! Indexers (the AnchorNet backend) subscribe to these events to keep an
//! off-chain view of pool liquidity in sync with on-chain state.

use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Emitted once when the contract is initialized. Topics: `("init",)`.
pub fn initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("init"),), admin.clone());
}

/// Emitted when the administrator changes. Topics: `("admin",)`.
pub fn admin_changed(env: &Env, new_admin: &Address) {
    env.events()
        .publish((symbol_short!("admin"),), new_admin.clone());
}

/// Emitted when an admin transfer is proposed. Topics: `("propose",)`.
pub fn admin_proposed(env: &Env, candidate: &Address) {
    env.events()
        .publish((symbol_short!("propose"),), candidate.clone());
}

/// Emitted when an anchor is registered. Topics: `("anchor", anchor)`.
pub fn anchor_registered(env: &Env, anchor: &Address) {
    env.events()
        .publish((symbol_short!("anchor"), anchor.clone()), ());
}

/// Emitted when liquidity is provided. Topics: `("provide", provider, asset)`.
pub fn liquidity_provided(env: &Env, provider: &Address, asset: &Symbol, amount: i128) {
    env.events().publish(
        (symbol_short!("provide"), provider.clone(), asset.clone()),
        amount,
    );
}

/// Emitted when liquidity is withdrawn. Topics: `("withdraw", provider, asset)`.
pub fn liquidity_withdrawn(env: &Env, provider: &Address, asset: &Symbol, amount: i128) {
    env.events().publish(
        (symbol_short!("withdraw"), provider.clone(), asset.clone()),
        amount,
    );
}

/// Emitted when the paused flag changes. Topics: `("paused",)`, data: `bool`.
pub fn paused_changed(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("paused"),), paused);
}

/// Emitted when the protocol fee changes. Topics: `("fee",)`, data: `u32` bps.
pub fn fee_changed(env: &Env, bps: u32) {
    env.events().publish((symbol_short!("fee"),), bps);
}

/// Emitted when an anchor is deregistered. Topics: `("deanchor", anchor)`.
pub fn anchor_removed(env: &Env, anchor: &Address) {
    env.events()
        .publish((symbol_short!("deanchor"), anchor.clone()), ());
}

/// Emitted when a settlement is opened. Topics: `("settle", anchor, asset)`.
pub fn settlement_opened(env: &Env, id: u64, anchor: &Address, asset: &Symbol) {
    env.events()
        .publish((symbol_short!("settle"), anchor.clone(), asset.clone()), id);
}

/// Emitted when a settlement is executed. Topics: `("executed", id)`.
pub fn settlement_executed(env: &Env, id: u64) {
    env.events().publish((symbol_short!("executed"), id), ());
}

/// Emitted when a settlement is cancelled. Topics: `("cancelled", id)`.
pub fn settlement_cancelled(env: &Env, id: u64) {
    env.events().publish((symbol_short!("cancelled"), id), ());
}

/// Emitted when an anchor's fee waiver flag changes. Topics:
/// `("waiver", anchor)`, data: `bool`.
pub fn fee_waiver_changed(env: &Env, anchor: &Address, waived: bool) {
    env.events()
        .publish((symbol_short!("waiver"), anchor.clone()), waived);
}

/// Emitted when accrued fees are collected. Topics: `("collect", asset)`.
pub fn fees_collected(env: &Env, asset: &Symbol, amount: i128) {
    env.events()
        .publish((symbol_short!("collect"), asset.clone()), amount);
}

/// Emitted when the settlement expiry window changes. Topics:
/// `("expiry",)`, data: `u32` ledgers.
pub fn settlement_expiry_changed(env: &Env, ledgers: u32) {
    env.events().publish((symbol_short!("expiry"),), ledgers);
}

/// Emitted when a timed-out settlement is reclaimed. Topics:
/// `("expired", id)`.
pub fn settlement_expired(env: &Env, id: u64) {
    env.events().publish((symbol_short!("expired"), id), ());
}

/// Emitted when an asset's minimum liquidity floor changes. Topics:
/// `("minliq", asset)`, data: `i128` floor.
pub fn min_liquidity_changed(env: &Env, asset: &Symbol, floor: i128) {
    env.events()
        .publish((symbol_short!("minliq"), asset.clone()), floor);
}

/// Emitted when the operator address changes. Topics: `("operator",)`.
pub fn operator_changed(env: &Env, operator: &Address) {
    env.events()
        .publish((symbol_short!("operator"),), operator.clone());
}

/// Emitted when an asset's maximum settlement amount changes. Topics:
/// `("maxamt", asset)`, data: `i128` amount.
pub fn max_settlement_amount_changed(env: &Env, asset: &Symbol, amount: i128) {
    env.events()
        .publish((symbol_short!("maxamt"), asset.clone()), amount);
}

/// Emitted when an asset's fee override is set. Topics: `("assetfee",
/// asset)`, data: `u32` bps.
pub fn asset_fee_changed(env: &Env, asset: &Symbol, bps: u32) {
    env.events()
        .publish((symbol_short!("assetfee"), asset.clone()), bps);
}

/// Emitted when an asset's fee override is cleared, reverting it to the
/// global fee. Topics: `("feeclear", asset)`.
pub fn asset_fee_cleared(env: &Env, asset: &Symbol) {
    env.events()
        .publish((symbol_short!("feeclear"), asset.clone()), ());
}
