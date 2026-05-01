# Development Scripts

Quick scripts for Rustphy frontend development.

## Quick Start

```bash
# One command to start everything
make dev
```

This will:
- ✅ Start HTTP server on `http://localhost:8080`
- ✅ Watch Rust files for changes
- ✅ Auto-rebuild WASM when you edit code
- ✅ Open `http://localhost:8080/playground.html` in your browser

## Individual Scripts

### `dev.sh` - Full Dev Environment
Starts server + auto-rebuild watch mode.

```bash
./scripts/dev.sh
```

### `watch-and-build.sh` - Watch Only
Just watches and rebuilds (no server).

```bash
./scripts/watch-and-build.sh
```

## Makefile Commands

```bash
make dev           # Start dev environment
make build-wasm    # Build WASM once
make serve         # Just start server
make clean         # Clean build artifacts
make install-tools # Install cargo-watch and wasm-pack
```

## Manual Workflow

If you prefer to do it manually:

```bash
# Terminal 1: Start server
cd frontend
python3 -m http.server 8080

# Terminal 2: Build WASM (run after each change)
wasm-pack build --target web --out-dir frontend/pkg
```

## Requirements

- `wasm-pack` - Install with `cargo install wasm-pack`
- `cargo-watch` - Install with `cargo install cargo-watch` (for auto-rebuild)
- `python3` - For HTTP server (or use any static file server)

## Workflow

1. Edit Rust code in `src/`
2. `cargo-watch` detects changes
3. Automatically runs `wasm-pack build`
4. Refresh browser to see changes

**Note:** Browser needs manual refresh. For auto-refresh, add a live-reload tool like [browser-sync](https://browsersync.io/).
