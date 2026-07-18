#!/usr/bin/env bash
# Build script for uwasm-toolbox.
# Compiles Rust to WASM via wasm-pack and places output in www/pkg/.
set -e
echo "Building uwasm-toolbox WASM..."
RUSTFLAGS="-C target-feature=+simd128" wasm-pack build --target web --out-dir www/pkg --release
echo ""
echo "Done. Serve the frontend with:"
echo "  python -m http.server -d www 8080"
echo "  # then open http://localhost:8080"
