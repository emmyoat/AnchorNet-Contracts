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
- `src/test.rs` – unit tests
- `Cargo.toml` – dependencies and crate config

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
