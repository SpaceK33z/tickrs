# typed: false
# frozen_string_literal: true

# Homebrew formula for tickrs - TickTick CLI for AI agents
class Tickrs < Formula
  desc "AI agent-optimized CLI for TickTick task management"
  homepage "https://github.com/SpaceK33z/tickrs"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/SpaceK33z/tickrs/releases/download/v#{version}/tickrs-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/SpaceK33z/tickrs/releases/download/v#{version}/tickrs-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    url "https://github.com/SpaceK33z/tickrs/releases/download/v#{version}/tickrs-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "tickrs"
  end

  test do
    assert_match "tickrs #{version}", shell_output("#{bin}/tickrs version")
  end
end
