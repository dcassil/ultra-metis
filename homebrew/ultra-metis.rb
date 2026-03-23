# typed: false
# frozen_string_literal: true

# Homebrew formula for ultra-metis
# Install: brew tap dcassil/ultra-metis && brew install ultra-metis
#
# This formula is auto-updated by the release workflow in the ultra-metis repo.
# Do not edit SHA256 hashes or version numbers manually.

class UltraMetis < Formula
  desc "Repo-native AI engineering orchestration system"
  homepage "https://github.com/dcassil/ultra-metis"
  license "Apache-2.0"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/dcassil/ultra-metis/releases/download/v#{version}/ultra-metis-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/dcassil/ultra-metis/releases/download/v#{version}/ultra-metis-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  def install
    bin.install "ultra-metis"
    bin.install "ultra-metis-mcp"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ultra-metis --version")
  end
end
