#!/bin/bash
set -e

echo "Building WASM..."
cargo build --target wasm32-unknown-unknown -p client --release

echo "Generating JS bindings..."
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/client.wasm

echo "Done! Serve web/ with a local server:"
echo "  cd web && python3 -m http.server 8000"
echo "  Then open http://localhost:8000"
