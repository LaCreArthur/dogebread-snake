#!/bin/bash
set -e

echo "Building WASM..."
~/.cargo/bin/cargo build -p client --lib --target wasm32-unknown-unknown --release

echo "Generating bindings..."
~/.cargo/bin/wasm-bindgen --out-dir web --target web target/wasm32-unknown-unknown/release/client.wasm

echo ""
echo "✓ Build complete!"
echo ""
echo "To test locally:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
