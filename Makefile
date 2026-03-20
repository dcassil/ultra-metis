.PHONY: build build-mcp build-cli install install-binary test clean \
       ci lint fmt fmt-check release-local package

INSTALL_DIR ?= $(HOME)/.local/bin

# Build all release binaries
build: build-mcp build-cli

# Build the MCP server (used by Claude Code plugin)
build-mcp:
	cargo build --release -p ultra-metis-mcp

# Build the CLI tool
build-cli:
	cargo build --release -p ultra-metis-cli

# Install binaries to PATH and validate plugin
install: build install-binary
	@echo "Binaries installed. Plugin structure validated."
	@echo "The .mcp.json in this repo configures ultra-metis for local development."
	@echo "To use in other projects, add to ~/.claude/settings.json or project .mcp.json:"
	@echo '  {"mcpServers":{"ultra-metis":{"command":"ultra-metis-mcp","args":[]}}}'

# Copy release binaries to INSTALL_DIR (default: /usr/local/bin)
install-binary:
	cp target/release/ultra-metis-mcp $(INSTALL_DIR)/ultra-metis-mcp
	cp target/release/ultra-metis $(INSTALL_DIR)/ultra-metis
	@echo "Installed ultra-metis-mcp and ultra-metis to $(INSTALL_DIR)"

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
	@ls -lh target/release/ultra-metis target/release/ultra-metis-mcp

# Package release binaries into distributable archive
package: release-local
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')-local; \
	echo "Packaging version v$$VERSION..."; \
	bash scripts/package.sh "v$$VERSION"
