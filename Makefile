.PHONY: test-quick capture test test-visual test-all build-wasm test-e2e lint fmt

# Fast: unit tests + simulation (<2s)
test-quick:
	cargo test --workspace

# Capture AUTO_TEST screenshots (~5s, needs GPU)
capture:
	AUTO_TEST=1 cargo run -p client --release

# Quick tests + screenshot validation if test-output/ exists
test: test-quick
	@if [ -d "test-output" ]; then \
		cargo test -p client --test screenshot_validation; \
	else \
		echo "Skipping screenshot validation (no test-output/). Run 'make capture' first."; \
	fi

# Capture + validate
test-visual: capture
	cargo test -p client --test screenshot_validation

# Build WASM + generate JS bindings
build-wasm:
	cargo build -p client --lib --target wasm32-unknown-unknown --release
	wasm-bindgen --out-dir web --target web target/wasm32-unknown-unknown/release/client.wasm

# Browser E2E tests (requires WASM build in web/)
test-e2e:
	cd e2e && bun install && bunx playwright install chromium && bunx playwright test

# Everything: unit + simulation + screenshot + WASM + E2E
test-all: test-quick
	@if [ -d "test-output" ]; then \
		cargo test -p client --test screenshot_validation; \
	fi
	$(MAKE) build-wasm
	$(MAKE) test-e2e

# Lint
lint:
	cargo clippy --workspace -- -D warnings

# Format check
fmt:
	cargo fmt --check
