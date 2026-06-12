.PHONY: build test fmt fmt-check wasm clean

# Build the contract for native testing.
build:
	cargo build

# Run the unit test suite.
test:
	cargo test

# Format the code in place.
fmt:
	cargo fmt --all

# Verify formatting (used in CI).
fmt-check:
	cargo fmt --all -- --check

# Build the optimized wasm artifact for deployment.
wasm:
	cargo build --target wasm32-unknown-unknown --release

# Remove build artifacts.
clean:
	cargo clean
