class WhoopCli < Formula
  desc "WHOOP fitness dashboard for the terminal"
  homepage "https://github.com/idossha/whoop-cli"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/idossha/whoop-cli/releases/download/v1.0.0/whoop-macos"
      sha256 "PLACEHOLDER_SHA256_INTEL"
    else
      url "https://github.com/idossha/whoop-cli/releases/download/v1.0.0/whoop-macos"
      sha256 "PLACEHOLDER_SHA256_ARM"
    end
  end

  def install
    bin.install "whoop-macos" => "whoop"
  end

  test do
    system "#{bin}/whoop", "--version"
  end
end
