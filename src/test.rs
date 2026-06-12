use crate::{AnchornetContract, AnchornetContractClient, Error};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

fn setup(env: &Env) -> (AnchornetContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, AnchornetContract);
    let client = AnchornetContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    (client, admin)
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
