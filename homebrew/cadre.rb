# typed: false
# frozen_string_literal: true

# Homebrew formula for cadre
# Install: brew tap dcassil/cadre && brew install cadre
#
# This formula is auto-updated by the release workflow in the cadre repo.
# Do not edit SHA256 hashes or version numbers manually.

class Cadre < Formula
  desc "Repo-native AI engineering orchestration system"
  homepage "https://github.com/dcassil/cadre"
  license "Apache-2.0"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/dcassil/cadre/releases/download/v#{version}/cadre-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/dcassil/cadre/releases/download/v#{version}/cadre-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  def install
    bin.install "cadre"
    bin.install "cadre-mcp"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cadre --version")
  end
end
