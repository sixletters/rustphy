.PHONY: help dev build-wasm serve clean install-tools

help:
	@echo "🦀 Rustphy Development Commands"
	@echo ""
	@echo "  make dev          - Start dev environment (auto-rebuild + server)"
	@echo "  make build-wasm   - Build WASM package for frontend"
	@echo "  make serve        - Start HTTP server for frontend"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make install-tools - Install development dependencies"
	@echo ""

# One-command development (auto-rebuild + server)
dev:
	@./scripts/dev.sh

# Build WASM package
build-wasm:
	@echo "🔨 Building WASM package..."
	@wasm-pack build --target web --out-dir frontend/pkg
	@echo "✅ WASM built to frontend/pkg/"

# Start HTTP server
serve:
	@echo "🌐 Starting server at http://localhost:8080"
	@echo "📝 Open http://localhost:8080/playground.html"
	@cd frontend && python3 -m http.server 8080

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf frontend/pkg
	@echo "✅ Clean complete"

# Install development tools
install-tools:
	@echo "📦 Installing development tools..."
	@command -v wasm-pack || cargo install wasm-pack
	@command -v cargo-watch || cargo install cargo-watch
	@echo "✅ Tools installed"
