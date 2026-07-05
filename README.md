# anchornet-contracts

Soroban smart contracts for **AnchorNet** — the liquidity coordination network for Stellar anchors. This repo contains on-chain logic for liquidity pools, routing metadata, and settlement hooks.

## Overview

- **Stack:** Rust, [Soroban SDK](https://soroban.stellar.org/docs)
- **Network:** Stellar (Soroban)

## Prerequisites

- [Rust](https://rustup.rs/) (stable, with `rustfmt`)
- Optional: [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli) for deployment and local testing

## Setup

```bash
# Clone the repo (or use your fork)
git clone <repo-url>
cd anchornet-contracts

# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Check formatting, build, and test
cargo fmt --all -- --check
cargo build
cargo test
```

## Project structure

- `src/lib.rs` – contract entrypoint and public interface
- `src/error.rs` – error codes returned to clients
- `src/types.rs` – on-chain data types (`Pool`)
- `src/storage.rs` – storage keys and TTL-aware accessors
- `src/events.rs` – event publishing helpers
- `src/test.rs` – unit tests
- `Cargo.toml` – dependencies and crate config

## Contract interface

The `AnchornetContract` tracks per-asset liquidity pools funded by registered
anchors. The off-chain indexer subscribes to the emitted events to mirror pool
state.

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin)` | once | Set the contract administrator |
| `admin()` | – | Read the current administrator |
| `set_admin(new_admin)` | admin | Transfer administration in a single step |
| `propose_admin(candidate)` | admin | Propose `candidate` as the next administrator |
| `accept_admin(candidate)` | candidate | Accept a pending admin transfer |
| `pending_admin()` | – | Read the proposed next administrator, if any |
| `register_anchor(anchor)` | admin | Approve an anchor as a liquidity provider |
| `is_anchor(anchor)` | – | Check whether an address is registered |
| `list_anchors()` | – | Enumerate currently registered anchors |
| `anchor_count()` | – | Read the number of currently registered anchors |
| `provide_liquidity(provider, asset, amount)` | provider | Add liquidity to a pool |
| `withdraw_liquidity(provider, asset, amount)` | provider | Remove liquidity from a pool |
| `deregister_anchor(anchor)` | admin | Remove an anchor from the approved set |
| `pool(asset)` | – | Read aggregate pool state |
| `total_liquidity(asset)` | – | Read total liquidity for an asset |
| `balance(provider, asset)` | – | Read a provider's balance |

### Admin & lifecycle

| Function | Auth | Description |
|----------|------|-------------|
| `pause()` / `unpause()` | admin | Halt or resume liquidity & settlement mutations |
| `is_paused()` | – | Read the paused flag |
| `set_fee(bps)` | admin | Set the protocol fee in basis points (max 1000) |
| `fee()` / `quote_fee(amount)` | – | Read the fee rate / preview a fee |
| `collect_fees(asset)` | admin | Collect accrued protocol fees for an asset |
| `fees_accrued(asset)` | – | Read uncollected fees for an asset |
| `version()` | – | Read the contract interface version |

### Settlement

| Function | Auth | Description |
|----------|------|-------------|
| `open_settlement(anchor, asset, amount)` | anchor | Reserve pool liquidity, returns a settlement id |
| `execute_settlement(id)` | admin | Finalize a settlement and accrue its fee |
| `cancel_settlement(id)` | anchor | Cancel and return reserved liquidity to the pool |
| `settlement(id)` | – | Read a settlement record |
| `settlement_count()` | – | Read the number of settlements |
| `list_settlements(start, limit)` | – | Page through settlements |
| `list_settlements_by_anchor(anchor, start, limit)` | – | Page through settlements opened by one anchor |

### Events

- `("init",)` – contract initialized
- `("admin",)` – administrator changed (via `set_admin` or `accept_admin`)
- `("propose",)` – admin transfer proposed
- `("anchor", anchor)` / `("deanchor", anchor)` – anchor registered / removed
- `("provide", provider, asset)` – liquidity provided
- `("withdraw", provider, asset)` – liquidity withdrawn
- `("paused",)` – paused flag changed
- `("fee",)` – protocol fee changed
- `("settle", anchor, asset)` – settlement opened
- `("executed", id)` / `("cancelled", id)` – settlement finalized / cancelled
- `("collect", asset)` – fees collected

## Commands

| Command | Description |
|--------|-------------|
| `cargo build` | Build the contract |
| `cargo test` | Run unit tests |
| `cargo fmt --all` | Format code |
| `cargo fmt --all -- --check` | Check formatting (CI) |

## Contributing

1. Fork the repo and create a branch from `main`.
2. Make changes; keep formatting with `cargo fmt --all`.
3. Run `cargo fmt --all -- --check`, `cargo build`, and `cargo test`.
4. Open a pull request. CI will run format check, build, and tests.

## License

MIT
