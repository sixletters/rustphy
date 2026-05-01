#!/bin/bash

# One-command development environment
# Starts server and watches for Rust file changes

echo "🚀 Starting Rustphy Development Environment"
echo ""

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

# Kill any existing python servers on port 8080
lsof -ti:8080 | xargs kill -9 2>/dev/null

# Start HTTP server in background
echo "🌐 Starting HTTP server on http://localhost:8080"
cd frontend && python3 -m http.server 8080 > /dev/null 2>&1 &
SERVER_PID=$!
cd ..

echo "📝 Server running at http://localhost:8080/playground.html"
echo "👀 Watching for Rust file changes..."
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Watch and rebuild on changes
cargo watch -w src -s 'wasm-pack build --target web --out-dir frontend/pkg'

# Cleanup on exit
trap "kill $SERVER_PID 2>/dev/null" EXIT
