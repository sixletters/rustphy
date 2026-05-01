#!/bin/bash

# Watch and rebuild WASM on file changes
# Usage: ./watch-and-build.sh

echo "🦀 Watching Rust files for changes..."
echo "📦 Will rebuild WASM to frontend/pkg/"
echo ""

# Function to build WASM
build_wasm() {
    echo "🔨 Building WASM..."
    wasm-pack build --target web --out-dir frontend/pkg
    if [ $? -eq 0 ]; then
        echo "✅ Build successful at $(date)"
    else
        echo "❌ Build failed at $(date)"
    fi
    echo ""
}

# Initial build
build_wasm

# Watch for changes (requires fswatch)
# Install with: brew install fswatch
if command -v fswatch &> /dev/null; then
    fswatch -o src/ | while read; do
        build_wasm
    done
else
    echo "❌ fswatch not found. Install with: brew install fswatch"
    echo "Or manually rebuild with: wasm-pack build --target web --out-dir frontend/pkg"
fi
