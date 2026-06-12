//! AnchorNet Soroban smart contracts.
//!
//! This crate contains on-chain logic for the AnchorNet liquidity coordination
//! network (liquidity pools, routing metadata, settlement hooks).

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod error;
mod events;
mod storage;
mod types;

pub use error::Error;
use types::Pool;

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
        events::anchor_registered(&env, &anchor);
        Ok(())
    }

    /// Returns `true` if `anchor` is a registered liquidity provider.
    pub fn is_anchor(env: Env, anchor: Address) -> bool {
        storage::is_anchor(&env, &anchor)
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
}

#[cfg(test)]
mod test;
