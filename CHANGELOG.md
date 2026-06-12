# Changelog

All notable changes to the AnchorNet contracts are documented here.

## [0.2.0]

### Added

- **Admin & lifecycle:** `pause` / `unpause` with an `is_paused` view; all
  liquidity and settlement mutations are blocked while paused.
- **Protocol fees:** configurable fee in basis points (`set_fee`, capped at
  10%), `fee` / `quote_fee` views, per-asset accrual, and admin `collect_fees`.
- **Settlement engine:** `open_settlement` reserves pool liquidity and returns
  an id; `execute_settlement` finalizes and accrues the fee; `cancel_settlement`
  returns reserved liquidity. Query via `settlement`, `settlement_exists`,
  `settlement_count`, and paginated `list_settlements`.
- **Anchor lifecycle:** `deregister_anchor` to remove an approved anchor.
- **Introspection:** `version` and `is_initialized` getters.
- Events for pause, fee changes, settlement lifecycle, and fee collection.

## [0.1.0]

### Added

- Initial liquidity pool registry: contract initialization, admin management,
  anchor registration, `provide_liquidity` / `withdraw_liquidity`, and pool /
  balance queries.
