//! Error types returned by the AnchorNet contract.

use soroban_sdk::contracterror;

/// Errors that can be returned by the AnchorNet liquidity contract.
///
/// Each variant maps to a stable `u32` code so that off-chain clients can
/// match on the error regardless of SDK version.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// The caller is not authorized to perform this action.
    NotAuthorized = 3,
    /// The anchor is already registered.
    AnchorAlreadyRegistered = 4,
    /// The anchor is not registered.
    AnchorNotRegistered = 5,
    /// The supplied amount must be greater than zero.
    InvalidAmount = 6,
    /// The pool does not hold enough liquidity for this operation.
    InsufficientLiquidity = 7,
    /// No pool exists for the requested asset.
    PoolNotFound = 8,
}
