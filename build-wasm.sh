#!/bin/bash
set -e

echo "Building WASM..."
~/.cargo/bin/cargo build -p client --lib --target wasm32-unknown-unknown --release

echo "Generating bindings..."
~/.cargo/bin/wasm-bindgen --out-dir web --target web target/wasm32-unknown-unknown/release/client.wasm

echo "Optimizing WASM with wasm-opt..."
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int web/client_bg.wasm -o web/client_bg.wasm
    echo "✓ WASM optimization complete"
else
    echo "⚠ wasm-opt not found. Install via: brew install binaryen"
fi

echo ""
echo "✓ Build complete!"
echo ""
WASM_SIZE=$(du -h web/client_bg.wasm | cut -f1)
echo "WASM size: $WASM_SIZE"
echo ""
echo "To test locally:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
