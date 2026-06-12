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
| `set_admin(new_admin)` | admin | Transfer administration |
| `register_anchor(anchor)` | admin | Approve an anchor as a liquidity provider |
| `is_anchor(anchor)` | – | Check whether an address is registered |
| `provide_liquidity(provider, asset, amount)` | provider | Add liquidity to a pool |
| `withdraw_liquidity(provider, asset, amount)` | provider | Remove liquidity from a pool |
| `pool(asset)` | – | Read aggregate pool state |
| `total_liquidity(asset)` | – | Read total liquidity for an asset |
| `balance(provider, asset)` | – | Read a provider's balance |

### Events

- `("init",)` – contract initialized
- `("anchor", anchor)` – anchor registered
- `("provide", provider, asset)` – liquidity provided
- `("withdraw", provider, asset)` – liquidity withdrawn

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
