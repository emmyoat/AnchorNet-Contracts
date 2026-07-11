//! On-chain data types for the AnchorNet liquidity contract.

use soroban_sdk::{contracttype, Address, Symbol};

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

/// Lifecycle state of a settlement request.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementStatus {
    /// Liquidity has been reserved against the pool, awaiting execution.
    Pending,
    /// The settlement has been executed and liquidity released to the anchor.
    Executed,
    /// The settlement was cancelled and reserved liquidity returned to the pool.
    Cancelled,
    /// The settlement timed out before execution and its reserved liquidity
    /// was reclaimed back to the pool via `cancel_expired_settlement`.
    Expired,
}

/// A request to draw `amount` of `asset` liquidity for cross-anchor settlement.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// Monotonic identifier assigned by the contract.
    pub id: u64,
    /// Anchor that requested the settlement.
    pub anchor: Address,
    /// Asset being settled.
    pub asset: Symbol,
    /// Gross amount reserved from the pool.
    pub amount: i128,
    /// Protocol fee withheld from the amount.
    pub fee: i128,
    /// Current lifecycle state.
    pub status: SettlementStatus,
    /// Ledger sequence number at which the settlement was opened, used to
    /// determine expiry via the contract-wide settlement expiry window.
    pub opened_at: u32,
}
