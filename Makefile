.PHONY: build build-mcp build-cli install install-binary test clean \
       ci lint lint-shell fmt fmt-check release-local package \
       build-desktop dev-desktop \
       start-api start-runner start-dashboard start-all

INSTALL_DIR ?= $(HOME)/.local/bin

# Build all release binaries
build: build-mcp build-cli

# Build the MCP server (used by Claude Code plugin)
build-mcp:
	cargo build --release -p cadre-mcp

# Build the CLI tool
build-cli:
	cargo build --release -p cadre-cli

# Install binaries to PATH and validate plugin
install: build install-binary
	@echo "Binaries installed. Plugin structure validated."
	@echo "The .mcp.json in this repo configures cadre for local development."
	@echo "To use in other projects, add to ~/.claude/settings.json or project .mcp.json:"
	@echo '  {"mcpServers":{"cadre":{"command":"cadre-mcp","args":[]}}}'

# Copy release binaries to INSTALL_DIR (default: /usr/local/bin)
install-binary:
	cp target/release/cadre-mcp $(INSTALL_DIR)/cadre-mcp
	cp target/release/cadre $(INSTALL_DIR)/cadre
	@echo "Installed cadre-mcp and cadre to $(INSTALL_DIR)"

# Run all tests
test:
	cargo test --workspace

# Clean build artifacts
clean:
	cargo clean
	rm -rf dist/

# --------------------------------------------------------------------------
# Local network services
# --------------------------------------------------------------------------

# Start the control API (port 3000)
start-api:
	apps/control-api/start.sh

# Start the machine runner (connects to API on localhost:3000)
start-runner:
	apps/machine-runner/start.sh

# Start the dashboard dev server (port 5173, LAN-accessible)
start-dashboard:
	apps/control-dashboard/start.sh

# Start all services (API + runner in background, dashboard in foreground)
start-all:
	@echo "Starting control-api..."
	@apps/control-api/start.sh &
	@sleep 3
	@echo "Starting machine-runner..."
	@apps/machine-runner/start.sh &
	@sleep 2
	@echo "Starting dashboard..."
	@LAN_IP=$$(ipconfig getifaddr en0 2>/dev/null || echo localhost); \
	echo ""; \
	echo "Dashboard: http://$$LAN_IP:5173"; \
	echo "API:       http://$$LAN_IP:3000"; \
	echo ""
	@apps/control-dashboard/start.sh

# --------------------------------------------------------------------------
# CI / Quality targets
# --------------------------------------------------------------------------

# Run the full CI suite locally (matches GitHub Actions CI workflow)
ci: test lint lint-shell fmt-check
	@echo "All CI checks passed."

# Run shellcheck on project shell scripts (excludes vendor/)
lint-shell:
	@echo "Running shellcheck on project shell scripts..."
	@find plugins/cadre benchmarks scripts tests -name '*.sh' -print0 | xargs -0 shellcheck -S warning

# Run clippy with warnings-as-errors (matches CI)
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Auto-format all Rust code
fmt:
	cargo fmt --all

# Check formatting without modifying files (matches CI)
fmt-check:
	cargo fmt --all -- --check

# --------------------------------------------------------------------------
# Release / Packaging targets
# --------------------------------------------------------------------------

# Build release binaries for the current platform
release-local: build
	@echo "Release binaries built:"
	@ls -lh target/release/cadre target/release/cadre-mcp

# --------------------------------------------------------------------------
# Desktop app targets
# --------------------------------------------------------------------------

# Build the Tauri desktop app for the current platform
build-desktop:
	cd apps/runner-desktop && npm ci && npm run tauri build

# Run the Tauri desktop app in dev mode
dev-desktop:
	cd apps/runner-desktop && npm run tauri dev

# Package release binaries into distributable archive
package: release-local
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')-local; \
	echo "Packaging version v$$VERSION..."; \
	bash scripts/package.sh "v$$VERSION"
