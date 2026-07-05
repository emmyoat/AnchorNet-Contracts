# Changelog

All notable changes to the AnchorNet contracts are documented here.

## [0.3.0]

### Added

- **Anchor enumeration:** `list_anchors` returns every currently registered
  anchor in registration order; `anchor_count` reads how many are active.
  Deregistered anchors are excluded but re-registration does not duplicate an
  anchor in the list.
- **Two-step admin transfer:** `propose_admin` / `accept_admin` offer a safer
  alternative to the existing single-step `set_admin`, requiring the proposed
  address to explicitly accept before control changes. `pending_admin` reads
  the outstanding proposal, if any.
- **Settlement queries:** `list_settlements_by_anchor` pages through the
  settlements opened by a single anchor.
- **Events:** `set_admin` (and the new `accept_admin`) now emit an `("admin",)`
  event on administrator change, closing a gap where admin transfers were
  previously silent; `propose_admin` emits `("propose",)`.

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
