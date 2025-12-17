class Ricecoder < Formula
  desc "Terminal-first, spec-driven coding assistant that understands your project before generating code"
  homepage "https://github.com/moabualruz/ricecoder"
  version "0.1.72"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/moabualruz/ricecoder/releases/download/v#{version}/ricecoder-#{version}-macos-arm64.tar.gz"
      sha256 "TODO: Add SHA256 checksum for ARM64 macOS binary"
    else
      url "https://github.com/moabualruz/ricecoder/releases/download/v#{version}/ricecoder-#{version}-macos-x86_64.tar.gz"
      sha256 "TODO: Add SHA256 checksum for x86_64 macOS binary"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/moabualruz/ricecoder/releases/download/v#{version}/ricecoder-#{version}-linux-arm64.tar.gz"
      sha256 "TODO: Add SHA256 checksum for ARM64 Linux binary"
    else
      url "https://github.com/moabualruz/ricecoder/releases/download/v#{version}/ricecoder-#{version}-linux-x86_64.tar.gz"
      sha256 "TODO: Add SHA256 checksum for x86_64 Linux binary"
    end
  end

  def install
    bin.install "ricecoder" => "rice"
  end

  test do
    system "#{bin}/rice", "--version"
  end
end