use crate::{AnchornetContract, AnchornetContractClient, Error, SettlementStatus};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    vec, Address, Env, IntoVal, Symbol,
};

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
fn test_set_admin_emits_admin_changed_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let new_admin = Address::generate(&env);

    client.initialize(&admin);
    client.set_admin(&new_admin);

    // `events().all()` reflects the most recent top-level invocation, i.e.
    // just the `set_admin` call.
    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                client.address.clone(),
                (symbol_short!("admin"),).into_val(&env),
                new_admin.into_val(&env),
            ),
        ]
    );
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

#[test]
fn test_open_settlement_reserves_liquidity() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%

    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(id, 1);
    assert_eq!(client.settlement_count(), 1);

    // Reserved liquidity leaves the available pool.
    assert_eq!(client.total_liquidity(&asset), 600);

    let settlement = client.settlement(&id);
    assert_eq!(settlement.amount, 400);
    assert_eq!(settlement.fee, 4); // 1% of 400
    assert_eq!(settlement.status, SettlementStatus::Pending);
}

#[test]
fn test_open_settlement_rejects_insufficient_liquidity() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 100);

    let err = client
        .try_open_settlement(&anchor, &asset, &500)
        .err()
        .unwrap()
        .unwrap();

    assert_eq!(err, Error::InsufficientLiquidity);
}

#[test]
fn test_open_settlement_rejects_unregistered() {
    let env = Env::default();
    let (client, _admin, _anchor, asset) = funded(&env, 1_000);
    let stranger = Address::generate(&env);

    let err = client
        .try_open_settlement(&stranger, &asset, &100)
        .err()
        .unwrap()
        .unwrap();

    assert_eq!(err, Error::AnchorNotRegistered);
}

#[test]
fn test_execute_settlement_accrues_fee() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%
    let id = client.open_settlement(&anchor, &asset, &400);

    client.execute_settlement(&id);

    assert_eq!(client.settlement(&id).status, SettlementStatus::Executed);
    assert_eq!(client.fees_accrued(&asset), 4);
    // Reserved liquidity stays out of the pool after execution.
    assert_eq!(client.total_liquidity(&asset), 600);
}

#[test]
fn test_cancel_settlement_returns_liquidity() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(client.total_liquidity(&asset), 600);

    client.cancel_settlement(&id);

    assert_eq!(client.settlement(&id).status, SettlementStatus::Cancelled);
    // Reserved liquidity is returned to the pool.
    assert_eq!(client.total_liquidity(&asset), 1_000);
    assert_eq!(client.fees_accrued(&asset), 0);
}

#[test]
fn test_settlement_not_found() {
    let env = Env::default();
    let (client, _admin, _anchor, _asset) = funded(&env, 1_000);

    let err = client.try_settlement(&99).err().unwrap().unwrap();
    assert_eq!(err, Error::SettlementNotFound);
}

#[test]
fn test_execute_twice_fails() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    let id = client.open_settlement(&anchor, &asset, &200);
    client.execute_settlement(&id);

    let err = client.try_execute_settlement(&id).err().unwrap().unwrap();
    assert_eq!(err, Error::InvalidSettlementState);
}

#[test]
fn test_cancel_executed_fails() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    let id = client.open_settlement(&anchor, &asset, &200);
    client.execute_settlement(&id);

    let err = client.try_cancel_settlement(&id).err().unwrap().unwrap();
    assert_eq!(err, Error::InvalidSettlementState);
}

#[test]
fn test_collect_fees_resets_balance() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%
    let id = client.open_settlement(&anchor, &asset, &500);
    client.execute_settlement(&id);
    assert_eq!(client.fees_accrued(&asset), 5);

    let collected = client.collect_fees(&asset);
    assert_eq!(collected, 5);
    assert_eq!(client.fees_accrued(&asset), 0);
}

#[test]
fn test_collect_fees_without_accrual_fails() {
    let env = Env::default();
    let (client, _admin, _anchor, asset) = funded(&env, 1_000);

    let err = client.try_collect_fees(&asset).err().unwrap().unwrap();
    assert_eq!(err, Error::NoFeesToCollect);
}

#[test]
fn test_deregister_anchor_blocks_settlement() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    client.deregister_anchor(&anchor);
    assert!(!client.is_anchor(&anchor));

    let err = client
        .try_open_settlement(&anchor, &asset, &100)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::AnchorNotRegistered);
}

#[test]
fn test_deregister_unknown_anchor_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin);
    let err = client
        .try_deregister_anchor(&stranger)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::AnchorNotRegistered);
}

#[test]
fn test_quote_fee_preview() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    client.set_fee(&250); // 2.5%

    assert_eq!(client.quote_fee(&1_000), 25);

    let err = client.try_quote_fee(&0).err().unwrap().unwrap();
    assert_eq!(err, Error::InvalidAmount);
}

#[test]
fn test_zero_fee_when_unset() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(client.settlement(&id).fee, 0);
}

#[test]
fn test_settlement_ids_increment() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    let first = client.open_settlement(&anchor, &asset, &100);
    let second = client.open_settlement(&anchor, &asset, &100);

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(client.settlement_count(), 2);
}

#[test]
fn test_paused_blocks_open_settlement() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    client.pause();
    let err = client
        .try_open_settlement(&anchor, &asset, &100)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::Paused);
}

#[test]
fn test_list_settlements_pagination() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    for _ in 0..3 {
        client.open_settlement(&anchor, &asset, &100);
    }

    let all = client.list_settlements(&1, &10);
    assert_eq!(all.len(), 3);

    let page = client.list_settlements(&2, &10);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().id, 2);

    let limited = client.list_settlements(&1, &1);
    assert_eq!(limited.len(), 1);
}

#[test]
fn test_list_settlements_by_anchor_filters_other_anchors() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let usdc = symbol_short!("USDC");

    client.initialize(&admin);
    client.register_anchor(&a1);
    client.register_anchor(&a2);
    client.provide_liquidity(&a1, &usdc, &1_000);
    client.provide_liquidity(&a2, &usdc, &1_000);

    let s1 = client.open_settlement(&a1, &usdc, &100);
    let s2 = client.open_settlement(&a2, &usdc, &100);
    let s3 = client.open_settlement(&a1, &usdc, &100);

    let a1_settlements = client.list_settlements_by_anchor(&a1, &1, &10);
    assert_eq!(a1_settlements.len(), 2);
    assert_eq!(a1_settlements.get(0).unwrap().id, s1);
    assert_eq!(a1_settlements.get(1).unwrap().id, s3);

    let a2_settlements = client.list_settlements_by_anchor(&a2, &1, &10);
    assert_eq!(a2_settlements.len(), 1);
    assert_eq!(a2_settlements.get(0).unwrap().id, s2);
}

#[test]
fn test_list_settlements_by_anchor_respects_limit() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    for _ in 0..3 {
        client.open_settlement(&anchor, &asset, &100);
    }

    let limited = client.list_settlements_by_anchor(&anchor, &1, &2);
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_list_settlements_by_anchor_empty_for_unknown() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.open_settlement(&anchor, &asset, &100);
    let stranger = Address::generate(&env);

    assert_eq!(
        client.list_settlements_by_anchor(&stranger, &1, &10).len(),
        0
    );
}

#[test]
fn test_list_settlements_by_asset_filters_other_assets() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");
    let eurc = symbol_short!("EURC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.provide_liquidity(&anchor, &usdc, &1_000);
    client.provide_liquidity(&anchor, &eurc, &1_000);

    let s1 = client.open_settlement(&anchor, &usdc, &100);
    let s2 = client.open_settlement(&anchor, &eurc, &100);
    let s3 = client.open_settlement(&anchor, &usdc, &100);

    let usdc_settlements = client.list_settlements_by_asset(&usdc, &1, &10);
    assert_eq!(usdc_settlements.len(), 2);
    assert_eq!(usdc_settlements.get(0).unwrap().id, s1);
    assert_eq!(usdc_settlements.get(1).unwrap().id, s3);

    let eurc_settlements = client.list_settlements_by_asset(&eurc, &1, &10);
    assert_eq!(eurc_settlements.len(), 1);
    assert_eq!(eurc_settlements.get(0).unwrap().id, s2);
}

#[test]
fn test_list_settlements_by_asset_respects_limit() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    for _ in 0..3 {
        client.open_settlement(&anchor, &asset, &100);
    }

    let limited = client.list_settlements_by_asset(&asset, &1, &2);
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_list_settlements_by_asset_empty_for_unknown() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.open_settlement(&anchor, &asset, &100);
    let other = symbol_short!("EURC");

    assert_eq!(client.list_settlements_by_asset(&other, &1, &10).len(), 0);
}

#[test]
fn test_version() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    assert_eq!(client.version(), 3);
}

#[test]
fn test_settlement_exists() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    assert!(!client.settlement_exists(&1));
    let id = client.open_settlement(&anchor, &asset, &100);
    assert!(client.settlement_exists(&id));
}

#[test]
fn test_list_settlements_empty() {
    let env = Env::default();
    let (client, _admin, _anchor, _asset) = funded(&env, 1_000);

    assert_eq!(client.list_settlements(&1, &10).len(), 0);
    assert_eq!(client.list_settlements(&100, &10).len(), 0);
}

#[test]
fn test_open_settlement_rejects_non_positive_amount() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);

    let err = client
        .try_open_settlement(&anchor, &asset, &0)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::InvalidAmount);
}

#[test]
fn test_is_initialized() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    assert!(!client.is_initialized());
    client.initialize(&admin);
    assert!(client.is_initialized());
}

#[test]
fn test_fees_accumulate_across_settlements() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%

    let first = client.open_settlement(&anchor, &asset, &300);
    let second = client.open_settlement(&anchor, &asset, &200);
    client.execute_settlement(&first);
    client.execute_settlement(&second);

    // 1% of 300 + 1% of 200 = 3 + 2 = 5
    assert_eq!(client.fees_accrued(&asset), 5);
}

#[test]
fn test_fees_are_tracked_per_asset() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);
    let usdc = symbol_short!("USDC");
    let eurc = symbol_short!("EURC");

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.set_fee(&100);
    client.provide_liquidity(&anchor, &usdc, &1_000);
    client.provide_liquidity(&anchor, &eurc, &1_000);

    let s1 = client.open_settlement(&anchor, &usdc, &400);
    client.execute_settlement(&s1);

    assert_eq!(client.fees_accrued(&usdc), 4);
    assert_eq!(client.fees_accrued(&eurc), 0);
}

#[test]
fn test_propose_and_accept_admin_transfers_control() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let candidate = Address::generate(&env);

    client.initialize(&admin);
    client.propose_admin(&candidate);
    assert_eq!(client.pending_admin(), candidate);
    // Control does not change until the candidate explicitly accepts.
    assert_eq!(client.admin(), admin);

    client.accept_admin(&candidate);

    assert_eq!(client.admin(), candidate);
    let err = client.try_pending_admin().err().unwrap().unwrap();
    assert_eq!(err, Error::NoPendingAdmin);
}

#[test]
fn test_accept_admin_without_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let candidate = Address::generate(&env);

    client.initialize(&admin);
    let err = client.try_accept_admin(&candidate).err().unwrap().unwrap();

    assert_eq!(err, Error::NoPendingAdmin);
}

#[test]
fn test_accept_admin_wrong_candidate_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let candidate = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin);
    client.propose_admin(&candidate);
    let err = client.try_accept_admin(&stranger).err().unwrap().unwrap();

    assert_eq!(err, Error::NotPendingAdmin);
    // The original proposal is untouched by the rejected attempt.
    assert_eq!(client.pending_admin(), candidate);
}

#[test]
fn test_propose_admin_overwrites_prior_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.initialize(&admin);
    client.propose_admin(&first);
    client.propose_admin(&second);

    assert_eq!(client.pending_admin(), second);
    let err = client.try_accept_admin(&first).err().unwrap().unwrap();
    assert_eq!(err, Error::NotPendingAdmin);
}

#[test]
fn test_pending_admin_unset_by_default() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.initialize(&admin);
    let err = client.try_pending_admin().err().unwrap().unwrap();
    assert_eq!(err, Error::NoPendingAdmin);
}

#[test]
fn test_list_anchors_returns_registered_in_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.list_anchors(&0, &10).len(), 0);
    assert_eq!(client.anchor_count(), 0);

    client.register_anchor(&a1);
    client.register_anchor(&a2);

    let anchors = client.list_anchors(&0, &10);
    assert_eq!(anchors.len(), 2);
    assert_eq!(anchors.get(0).unwrap(), a1);
    assert_eq!(anchors.get(1).unwrap(), a2);
    assert_eq!(client.anchor_count(), 2);
}

#[test]
fn test_list_anchors_excludes_deregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.initialize(&admin);
    client.register_anchor(&a1);
    client.register_anchor(&a2);
    client.deregister_anchor(&a1);

    let anchors = client.list_anchors(&0, &10);
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors.get(0).unwrap(), a2);
    assert_eq!(client.anchor_count(), 1);
}

#[test]
fn test_list_anchors_reflects_reregistration() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let anchor = Address::generate(&env);

    client.initialize(&admin);
    client.register_anchor(&anchor);
    client.deregister_anchor(&anchor);
    assert_eq!(client.anchor_count(), 0);

    // Re-registering a previously removed anchor must not duplicate it in
    // the enumerated list.
    client.register_anchor(&anchor);
    let anchors = client.list_anchors(&0, &10);
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors.get(0).unwrap(), anchor);
}

#[test]
fn test_list_anchors_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    client.initialize(&admin);
    client.register_anchor(&a1);
    client.register_anchor(&a2);
    client.register_anchor(&a3);

    let page = client.list_anchors(&0, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap(), a1);
    assert_eq!(page.get(1).unwrap(), a2);

    let rest = client.list_anchors(&2, &10);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest.get(0).unwrap(), a3);

    let none = client.list_anchors(&3, &10);
    assert_eq!(none.len(), 0);
}

#[test]
fn test_list_anchors_pagination_skips_deregistered_without_counting() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);
    let a3 = Address::generate(&env);

    client.initialize(&admin);
    client.register_anchor(&a1);
    client.register_anchor(&a2);
    client.register_anchor(&a3);
    client.deregister_anchor(&a2);

    // Scanning from list index 0 with a limit of 2 must skip the
    // deregistered a2 (list index 1) without counting it toward the limit,
    // so both a1 and a3 are still returned.
    let page = client.list_anchors(&0, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap(), a1);
    assert_eq!(page.get(1).unwrap(), a3);
}

#[test]
fn test_fee_waiver_exempts_anchor_from_settlement_fee() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%

    client.set_fee_waiver(&anchor, &true);
    assert!(client.is_fee_waived(&anchor));

    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(client.settlement(&id).fee, 0);
}

#[test]
fn test_fee_waiver_toggle_off_restores_fee() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100); // 1%

    client.set_fee_waiver(&anchor, &true);
    client.set_fee_waiver(&anchor, &false);
    assert!(!client.is_fee_waived(&anchor));

    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(client.settlement(&id).fee, 4);
}

#[test]
fn test_set_fee_waiver_rejects_unregistered_anchor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin);
    let err = client
        .try_set_fee_waiver(&stranger, &true)
        .err()
        .unwrap()
        .unwrap();
    assert_eq!(err, Error::AnchorNotRegistered);
}

#[test]
fn test_fee_waiver_unset_by_default() {
    let env = Env::default();
    let (client, _admin, anchor, _asset) = funded(&env, 1_000);

    assert!(!client.is_fee_waived(&anchor));
}

#[test]
fn test_cancel_restores_liquidity_with_fee_set() {
    let env = Env::default();
    let (client, _admin, anchor, asset) = funded(&env, 1_000);
    client.set_fee(&100);

    let id = client.open_settlement(&anchor, &asset, &400);
    assert_eq!(client.total_liquidity(&asset), 600);

    client.cancel_settlement(&id);

    // The full reserved amount returns; fees are only accrued on execution.
    assert_eq!(client.total_liquidity(&asset), 1_000);
    assert_eq!(client.fees_accrued(&asset), 0);
}
