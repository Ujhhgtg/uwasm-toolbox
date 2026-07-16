#!/usr/bin/env bash
set -e
echo "Building uwasm-toolbox WASM..."
wasm-pack build --target web --out-dir www/pkg --release
echo ""
echo "Done. Serve the frontend with:"
echo "  python -m http.server -d www 8080"
echo "  # then open http://localhost:8080"
