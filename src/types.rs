//! On-chain data types for the AnchorNet liquidity contract.

use soroban_sdk::{contracttype, Symbol};

/// A liquidity pool for a single asset within AnchorNet.
///
/// Pools aggregate liquidity supplied by many providers so that the routing
/// layer can settle cross-anchor payments against a shared balance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    /// Asset symbol the pool holds liquidity for (e.g. `USDC`, `XLM`).
    pub asset: Symbol,
    /// Total liquidity currently provided across all providers.
    pub total: i128,
    /// Number of distinct providers contributing to this pool.
    pub providers: u32,
}

impl Pool {
    /// Creates an empty pool for `asset` with no liquidity and no providers.
    pub fn empty(asset: Symbol) -> Self {
        Pool {
            asset,
            total: 0,
            providers: 0,
        }
    }
}
