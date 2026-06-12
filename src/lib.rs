//! AnchorNet Soroban smart contracts.
//!
//! This crate contains on-chain logic for the AnchorNet liquidity coordination
//! network (liquidity pools, routing metadata, settlement hooks).

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Vec};

mod error;
mod events;
mod storage;
mod types;

use error::Error;
#[allow(unused_imports)]
use types::Pool;

const SYMBOL_GREETING: Symbol = symbol_short!("greeting");

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

    /// Returns a greeting; used to verify contract deployment and CI.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        let mut v: Vec<Symbol> = Vec::new(&env);
        v.push_back(SYMBOL_GREETING);
        v.push_back(to);
        v
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
