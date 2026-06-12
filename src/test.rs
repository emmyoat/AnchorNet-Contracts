use crate::{AnchornetContract, AnchornetContractClient, Error, SettlementStatus};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Symbol};

fn setup(env: &Env) -> (AnchornetContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, AnchornetContract);
    let client = AnchornetContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    (client, admin)
}

/// Initializes the contract, registers one anchor, and funds a pool.
/// Auths are mocked. Returns the client, admin, anchor and funded asset.
fn funded(env: &Env, liquidity: i128) -> (AnchornetContractClient<'_>, Address, Address, Symbol) {
    env.mock_all_auths();
    let (client, admin) = setup(env);
    let anchor = Address::generate(env);
    let asset = symbol_short!("USDC");
    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &asset, &liquidity);
    (client, admin, anchor, asset)
}

#[test]
fn test_initialize_sets_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.initialize(&admin);
    let err = client.try_initialize(&admin).err().unwrap().unwrap();

    assert_eq!(err, Error::AlreadyInitialized);
}

#[test]
fn test_register_anchor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);

    client.initialize(&admin);
    assert!(!client.is_anchor(&anchor));

    client.register_anchor(&anchor);
    assert!(client.is_anchor(&anchor));
}

#[test]
fn test_register_anchor_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);

    client.initialize(&admin);
    client.register_anchor(&anchor);
    let err = client.try_register_anchor(&anchor).err().unwrap().unwrap();

    assert_eq!(err, Error::AnchorAlreadyRegistered);
}

#[test]
fn test_provide_liquidity_updates_pool_and_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &usdc, &1_000);

    assert_eq!(client.total_liquidity(&usdc), 1_000);
    assert_eq!(client.balance(&anchor, &usdc), 1_000);

    let pool = client.pool(&usdc);
    assert_eq!(pool.total, 1_000);
    assert_eq!(pool.providers, 1);
}

#[test]
fn test_pool_aggregates_multiple_providers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&a1);
    client.register_anchor(&a2);
    client.provide_liquidity(&a1, &usdc, &600);
    client.provide_liquidity(&a2, &usdc, &400);

    let pool = client.pool(&usdc);
    assert_eq!(pool.total, 1_000);
    assert_eq!(pool.providers, 2);
}

#[test]
fn test_provide_liquidity_rejects_unregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    let err = client
        .try_provide_liquidity(&stranger, &usdc, &100)
        .err()
        .unwrap()
        .unwrap();

    assert_eq!(err, Error::AnchorNotRegistered);
}

#[test]
fn test_provide_liquidity_rejects_non_positive_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    let err = client
        .try_provide_liquidity(&anchor, &usdc, &0)
        .err()
        .unwrap()
        .unwrap();

    assert_eq!(err, Error::InvalidAmount);
}

#[test]
fn test_withdraw_reduces_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &usdc, &1_000);
    client.withdraw_liquidity(&anchor, &usdc, &400);

    assert_eq!(client.balance(&anchor, &usdc), 600);
    assert_eq!(client.total_liquidity(&usdc), 600);
}

#[test]
fn test_full_withdraw_drops_provider_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &usdc, &1_000);
    client.withdraw_liquidity(&anchor, &usdc, &1_000);

    let pool = client.pool(&usdc);
    assert_eq!(pool.total, 0);
    assert_eq!(pool.providers, 0);
}

#[test]
fn test_withdraw_insufficient_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &usdc, &100);
    let err = client
        .try_withdraw_liquidity(&anchor, &usdc, &500)
        .err()
        .unwrap()
        .unwrap();

    assert_eq!(err, Error::InsufficientLiquidity);
}

#[test]
fn test_pool_not_found() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    let err = client.try_pool(&usdc).err().unwrap().unwrap();

    assert_eq!(err, Error::PoolNotFound);
}

#[test]
fn test_unknown_balance_is_zero() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);

    assert_eq!(client.balance(&anchor, &usdc), 0);
    assert_eq!(client.total_liquidity(&usdc), 0);
}

#[test]
fn test_set_admin_transfers_control() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.initialize(&admin);
    client.set_admin(&new_admin);

    assert_eq!(client.admin(), new_admin);
}

#[test]
fn test_pause_and_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.initialize(&admin);
    assert!(!client.is_paused());

    client.pause();
    assert!(client.is_paused());

    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_paused_blocks_provide_and_withdraw() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    client.pause();

    let provide = client
        .try_provide_liquidity(&anchor, &asset, &100)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(provide, Error::Paused);

    let withdraw = client
        .try_withdraw_liquidity(&anchor, &asset, &100)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(withdraw, Error::Paused);
}

#[test]
fn test_set_fee_updates_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.initialize(&admin);
    assert_eq!(client.fee(), 0);

    client.set_fee(&25);
    assert_eq!(client.fee(), 25);
}

#[test]
fn test_set_fee_rejects_above_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.initialize(&admin);
    let err = client.try_set_fee(&1_001).err().unwrap().unwrap();

    assert_eq!(err, Error::InvalidFee);
}
