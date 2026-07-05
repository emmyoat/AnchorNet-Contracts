//! AnchorNet Soroban smart contracts.
//!
//! This crate contains on-chain logic for the AnchorNet liquidity coordination
//! network (liquidity pools, routing metadata, settlement hooks).

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

mod error;
mod events;
mod storage;
mod types;

pub use error::Error;
pub use types::{Pool, Settlement, SettlementStatus};

/// Maximum protocol fee that can be configured: 1000 bps (10%).
const MAX_FEE_BPS: u32 = 1_000;
/// Basis-points denominator.
const BPS_DENOMINATOR: i128 = 10_000;

/// The AnchorNet liquidity coordination contract.
///
/// Tracks per-asset liquidity pools funded by registered anchors so the
/// off-chain routing layer can settle cross-anchor payments against a shared,
/// auditable on-chain balance.
#[contract]
pub struct AnchornetContract;

#[contractimpl]
impl AnchornetContract {
    /// Initializes the contract and sets the administrator.
    ///
    /// Can only be called once; subsequent calls return
    /// [`Error::AlreadyInitialized`].
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        events::initialized(&env, &admin);
        Ok(())
    }

    /// Returns the contract interface version.
    pub fn version() -> u32 {
        2
    }

    /// Returns `true` if the contract has been initialized.
    pub fn is_initialized(env: Env) -> bool {
        storage::has_admin(&env)
    }

    /// Returns the current administrator address.
    pub fn admin(env: Env) -> Result<Address, Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }
        Ok(storage::get_admin(&env))
    }

    /// Transfers administration to `new_admin`. Requires authorization from the
    /// current administrator.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;
        storage::set_admin(&env, &new_admin);
        Ok(())
    }

    /// Pauses the contract, blocking liquidity and settlement mutations.
    /// Admin only.
    pub fn pause(env: Env) -> Result<(), Error> {
        Self::require_admin(&env)?;
        storage::set_paused(&env, true);
        events::paused_changed(&env, true);
        Ok(())
    }

    /// Resumes the contract after a pause. Admin only.
    pub fn unpause(env: Env) -> Result<(), Error> {
        Self::require_admin(&env)?;
        storage::set_paused(&env, false);
        events::paused_changed(&env, false);
        Ok(())
    }

    /// Returns `true` if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    /// Sets the protocol fee in basis points (max 1000 = 10%). Admin only.
    pub fn set_fee(env: Env, bps: u32) -> Result<(), Error> {
        Self::require_admin(&env)?;
        if bps > MAX_FEE_BPS {
            return Err(Error::InvalidFee);
        }
        storage::set_fee_bps(&env, bps);
        events::fee_changed(&env, bps);
        Ok(())
    }

    /// Returns the protocol fee in basis points.
    pub fn fee(env: Env) -> u32 {
        storage::get_fee_bps(&env)
    }

    /// Previews the protocol fee charged for settling `amount` at the current
    /// fee rate, without changing any state.
    pub fn quote_fee(env: Env, amount: i128) -> Result<i128, Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        Ok(amount * (storage::get_fee_bps(&env) as i128) / BPS_DENOMINATOR)
    }

    /// Collects the accrued protocol fees for `asset`, resetting the balance to
    /// zero and returning the collected amount. Admin only.
    pub fn collect_fees(env: Env, asset: Symbol) -> Result<i128, Error> {
        Self::require_admin(&env)?;
        let amount = storage::get_fees_accrued(&env, &asset);
        if amount == 0 {
            return Err(Error::NoFeesToCollect);
        }
        storage::set_fees_accrued(&env, &asset, 0);
        events::fees_collected(&env, &asset, amount);
        Ok(amount)
    }

    /// Registers `anchor` as an approved liquidity provider. Admin only.
    ///
    /// Returns [`Error::AnchorAlreadyRegistered`] if the anchor is already
    /// registered.
    pub fn register_anchor(env: Env, anchor: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;
        if storage::is_anchor(&env, &anchor) {
            return Err(Error::AnchorAlreadyRegistered);
        }
        storage::set_anchor(&env, &anchor);
        storage::remember_anchor(&env, &anchor);
        events::anchor_registered(&env, &anchor);
        Ok(())
    }

    /// Returns `true` if `anchor` is a registered liquidity provider.
    pub fn is_anchor(env: Env, anchor: Address) -> bool {
        storage::is_anchor(&env, &anchor)
    }

    /// Returns every currently registered anchor, in registration order.
    ///
    /// Anchors that have been [`deregister_anchor`](Self::deregister_anchor)ed
    /// are excluded.
    pub fn list_anchors(env: Env) -> Vec<Address> {
        let mut out = Vec::new(&env);
        for anchor in storage::get_anchor_list(&env).iter() {
            if storage::is_anchor(&env, &anchor) {
                out.push_back(anchor);
            }
        }
        out
    }

    /// Returns the number of currently registered anchors.
    pub fn anchor_count(env: Env) -> u32 {
        Self::list_anchors(env).len()
    }

    /// Removes `anchor` from the approved set. Admin only. Existing pool
    /// liquidity is unaffected; the anchor simply cannot open new positions.
    pub fn deregister_anchor(env: Env, anchor: Address) -> Result<(), Error> {
        Self::require_admin(&env)?;
        if !storage::is_anchor(&env, &anchor) {
            return Err(Error::AnchorNotRegistered);
        }
        storage::set_anchor_flag(&env, &anchor, false);
        events::anchor_removed(&env, &anchor);
        Ok(())
    }

    /// Provides `amount` of liquidity in `asset` from `provider`.
    ///
    /// The provider must be a registered anchor and must authorize the call.
    /// The pool's total and the provider's balance are increased by `amount`.
    pub fn provide_liquidity(
        env: Env,
        provider: Address,
        asset: Symbol,
        amount: i128,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_not_paused(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if !storage::is_anchor(&env, &provider) {
            return Err(Error::AnchorNotRegistered);
        }

        let mut pool = storage::get_pool(&env, &asset);
        let prior = storage::get_balance(&env, &provider, &asset);
        if prior == 0 {
            pool.providers += 1;
        }
        pool.total += amount;
        storage::set_pool(&env, &asset, &pool);
        storage::set_balance(&env, &provider, &asset, prior + amount);

        events::liquidity_provided(&env, &provider, &asset, amount);
        Ok(())
    }

    /// Withdraws `amount` of liquidity in `asset` back to `provider`.
    ///
    /// Requires authorization from `provider` and fails with
    /// [`Error::InsufficientLiquidity`] if the provider's balance is too low.
    pub fn withdraw_liquidity(
        env: Env,
        provider: Address,
        asset: Symbol,
        amount: i128,
    ) -> Result<(), Error> {
        provider.require_auth();
        Self::require_not_paused(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let prior = storage::get_balance(&env, &provider, &asset);
        if prior < amount {
            return Err(Error::InsufficientLiquidity);
        }

        let mut pool = storage::get_pool(&env, &asset);
        pool.total -= amount;
        let remaining = prior - amount;
        if remaining == 0 {
            pool.providers -= 1;
        }
        storage::set_pool(&env, &asset, &pool);
        storage::set_balance(&env, &provider, &asset, remaining);

        events::liquidity_withdrawn(&env, &provider, &asset, amount);
        Ok(())
    }

    /// Opens a settlement that reserves `amount` of `asset` liquidity for the
    /// requesting `anchor`. The reserved amount leaves the available pool and a
    /// [`SettlementStatus::Pending`] record is created. Returns the new id.
    pub fn open_settlement(
        env: Env,
        anchor: Address,
        asset: Symbol,
        amount: i128,
    ) -> Result<u64, Error> {
        anchor.require_auth();
        Self::require_not_paused(&env)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if !storage::is_anchor(&env, &anchor) {
            return Err(Error::AnchorNotRegistered);
        }

        let mut pool = storage::get_pool(&env, &asset);
        if pool.total < amount {
            return Err(Error::InsufficientLiquidity);
        }
        pool.total -= amount;
        storage::set_pool(&env, &asset, &pool);

        let fee = amount * (storage::get_fee_bps(&env) as i128) / BPS_DENOMINATOR;
        let id = storage::get_settlement_count(&env) + 1;
        storage::set_settlement_count(&env, id);
        storage::set_settlement(
            &env,
            &Settlement {
                id,
                anchor: anchor.clone(),
                asset: asset.clone(),
                amount,
                fee,
                status: SettlementStatus::Pending,
            },
        );

        events::settlement_opened(&env, id, &anchor, &asset);
        Ok(id)
    }

    /// Executes a pending settlement, accruing its fee for later collection.
    /// Admin only. The reserved liquidity is considered released to the anchor.
    pub fn execute_settlement(env: Env, id: u64) -> Result<(), Error> {
        Self::require_admin(&env)?;
        let mut settlement = storage::get_settlement(&env, id).ok_or(Error::SettlementNotFound)?;
        if settlement.status != SettlementStatus::Pending {
            return Err(Error::InvalidSettlementState);
        }

        let accrued = storage::get_fees_accrued(&env, &settlement.asset);
        storage::set_fees_accrued(&env, &settlement.asset, accrued + settlement.fee);

        settlement.status = SettlementStatus::Executed;
        storage::set_settlement(&env, &settlement);

        events::settlement_executed(&env, id);
        Ok(())
    }

    /// Cancels a pending settlement and returns the reserved liquidity to the
    /// pool. Requires authorization from the settlement's anchor.
    pub fn cancel_settlement(env: Env, id: u64) -> Result<(), Error> {
        let mut settlement = storage::get_settlement(&env, id).ok_or(Error::SettlementNotFound)?;
        settlement.anchor.require_auth();
        if settlement.status != SettlementStatus::Pending {
            return Err(Error::InvalidSettlementState);
        }

        let mut pool = storage::get_pool(&env, &settlement.asset);
        pool.total += settlement.amount;
        storage::set_pool(&env, &settlement.asset, &pool);

        settlement.status = SettlementStatus::Cancelled;
        storage::set_settlement(&env, &settlement);

        events::settlement_cancelled(&env, id);
        Ok(())
    }

    /// Returns the [`Pool`] for `asset`, or [`Error::PoolNotFound`] if no
    /// liquidity has ever been provided for it.
    pub fn pool(env: Env, asset: Symbol) -> Result<Pool, Error> {
        if !storage::has_pool(&env, &asset) {
            return Err(Error::PoolNotFound);
        }
        Ok(storage::get_pool(&env, &asset))
    }

    /// Returns the total liquidity available in `asset` across all providers.
    pub fn total_liquidity(env: Env, asset: Symbol) -> i128 {
        storage::get_pool(&env, &asset).total
    }

    /// Returns `provider`'s liquidity balance in `asset` (zero if none).
    pub fn balance(env: Env, provider: Address, asset: Symbol) -> i128 {
        storage::get_balance(&env, &provider, &asset)
    }

    /// Returns the settlement with `id`, or [`Error::SettlementNotFound`].
    pub fn settlement(env: Env, id: u64) -> Result<Settlement, Error> {
        storage::get_settlement(&env, id).ok_or(Error::SettlementNotFound)
    }

    /// Returns the number of settlements ever opened.
    pub fn settlement_count(env: Env) -> u64 {
        storage::get_settlement_count(&env)
    }

    /// Returns `true` if a settlement with `id` exists.
    pub fn settlement_exists(env: Env, id: u64) -> bool {
        storage::get_settlement(&env, id).is_some()
    }

    /// Returns up to `limit` settlements starting at id `start` (inclusive).
    /// Ids are assigned sequentially from 1; missing ids are skipped.
    pub fn list_settlements(env: Env, start: u64, limit: u32) -> Vec<Settlement> {
        let mut out = Vec::new(&env);
        let count = storage::get_settlement_count(&env);
        let mut id = if start == 0 { 1 } else { start };
        while id <= count && (out.len() as u32) < limit {
            if let Some(settlement) = storage::get_settlement(&env, id) {
                out.push_back(settlement);
            }
            id += 1;
        }
        out
    }

    /// Returns the accrued (uncollected) protocol fees for `asset`.
    pub fn fees_accrued(env: Env, asset: Symbol) -> i128 {
        storage::get_fees_accrued(&env, &asset)
    }
}

impl AnchornetContract {
    /// Requires the call to be authorized by the current administrator.
    fn require_admin(env: &Env) -> Result<(), Error> {
        if !storage::has_admin(env) {
            return Err(Error::NotInitialized);
        }
        let admin = storage::get_admin(env);
        admin.require_auth();
        Ok(())
    }

    /// Requires the contract to be active (not paused).
    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if storage::is_paused(env) {
            return Err(Error::Paused);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
