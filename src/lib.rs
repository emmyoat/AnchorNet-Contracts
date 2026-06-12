//! AnchorNet Soroban smart contracts.
//!
//! This crate contains on-chain logic for the AnchorNet liquidity coordination
//! network (liquidity pools, routing metadata, settlement hooks).

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol, Vec};

mod error;

#[allow(unused_imports)]
use error::Error;

const SYMBOL_GREETING: Symbol = symbol_short!("greeting");

/// Placeholder AnchorNet contract for liquidity coordination.
/// Extended in later phases with pool and settlement logic.
#[contract]
pub struct AnchornetContract;

#[contractimpl]
impl AnchornetContract {
    /// Returns a greeting; used to verify contract deployment and CI.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        let mut v: Vec<Symbol> = Vec::new(&env);
        v.push_back(SYMBOL_GREETING);
        v.push_back(to);
        v
    }
}

#[cfg(test)]
mod test;
