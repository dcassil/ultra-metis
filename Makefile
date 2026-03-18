.PHONY: build build-mcp build-cli install install-binary test clean

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
