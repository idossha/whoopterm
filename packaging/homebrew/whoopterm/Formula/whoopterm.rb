class Whoopterm < Formula
  desc "WHOOP fitness dashboard for the terminal"
  homepage "https://github.com/idossha/whoopterm"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/idossha/whoopterm/releases/download/v1.0.0/whoopterm-macos"
      sha256 "PLACEHOLDER_SHA256_INTEL"
    else
      url "https://github.com/idossha/whoopterm/releases/download/v1.0.0/whoopterm-macos"
      sha256 "PLACEHOLDER_SHA256_ARM"
    end
  end

  def install
    bin.install "whoopterm-macos" => "whoopterm"
  end

  test do
    system "#{bin}/whoopterm", "--version"
  end
end