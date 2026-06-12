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
