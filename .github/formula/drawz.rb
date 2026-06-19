# This formula is auto-updated by the release workflow via repository-dispatch.
# Place this in your homebrew-tap repo at Formula/drawz.rb

class Drawz < Formula
  desc "Rendering guarantee layer between AI agents and terminal display"
  homepage "https://github.com/OWNER/drawz"
  version "VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/OWNER/drawz/releases/download/vVERSION/drawz-aarch64-apple-darwin.tar.gz"
      sha256 "MAC_ARM_SHA256"
    else
      url "https://github.com/OWNER/drawz/releases/download/vVERSION/drawz-x86_64-apple-darwin.tar.gz"
      sha256 "MAC_INTEL_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/OWNER/drawz/releases/download/vVERSION/drawz-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "LINUX_ARM_SHA256"
    else
      url "https://github.com/OWNER/drawz/releases/download/vVERSION/drawz-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "LINUX_INTEL_SHA256"
    end
  end

  def install
    bin.install "drawz"
  end

  test do
    output = shell_output("echo '{\"type\":\"flow\",\"steps\":[\"A\",\"B\"]}' | #{bin}/drawz")
    assert_match "A", output
  end
end
