.PHONY: build build-mcp build-cli install install-binary test clean \
       ci lint fmt fmt-check release-local package

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
# CI / Quality targets
# --------------------------------------------------------------------------

# Run the full CI suite locally (matches GitHub Actions CI workflow)
ci: test lint fmt-check
	@echo "All CI checks passed."

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

# Package release binaries into distributable archive
package: release-local
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')-local; \
	echo "Packaging version v$$VERSION..."; \
	bash scripts/package.sh "v$$VERSION"
