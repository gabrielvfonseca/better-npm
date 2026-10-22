# typed: true
# frozen_string_literal: true

# Bnpm — JavaScript package manager
# This file can be used as a standalone formula for `brew install` from URL.
# Usage: brew install https://raw.githubusercontent.com/org/bnpm/main/homebrew/bnpm.rb

class Bnpm < Formula
  desc "JavaScript package manager — fast, secure, and reliable"
  homepage "https://github.com/org/bnpm"
  license "MIT"
  head "https://github.com/org/bnpm.git", branch: "main"

  stable do
    url "https://github.com/org/bnpm/archive/refs/tags/v0.0.0.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  livecheck do
    url :stable
    strategy :github_latest
  end

  depends_on "pkg-config" => :build
  depends_on "rust" => :build
  depends_on "openssl@3"
  depends_on "zstd"

  uses_from_macos "bzip2"
  uses_from_macos "curl"
  uses_from_macos "sqlite"
  uses_from_macos "xz"
  uses_from_macos "zlib"

  def install
    ENV["OPENSSL_NO_VENDOR"] = "1"
    ENV["OPENSSL_DIR"] = Formula["openssl@3"].opt_prefix

    system "cargo", "install", *std_cargo_args(path: "crates/cli")

    generate_completions
    generate_man_pages
  end

  def generate_completions
    bash_completion.install Utils::ShellCompletions.new(
      bin/"bnpm", "bash",
      shell_function: :bash
    ).output

    zsh_completion.install Utils::ShellCompletions.new(
      bin/"bnpm", "zsh",
      shell_function: :zsh
    ).output

    fish_completion.install Utils::ShellCompletions.new(
      bin/"bnpm", "fish",
      shell_function: :fish
    ).output
  end

  def generate_man_pages
    system bin/"bnpm", "man", "--output", buildpath/"man"
    man1.install Dir[buildpath/"man/**/*.1"]
  end

  def post_install
    (var/"bnpm/cache").mkpath
    (var/"bnpm/data").mkpath
  end

  def caveats
    <<~EOS
      Bnpm is installed as `bnpm`.

      Cache: #{var}/bnpm/cache
      Data:  #{var}/bnpm/data

      Quick start:
        bnpm help
        bnpm init my-project
        cd my-project && bnpm install
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/bnpm --version")
    assert_match "Usage:", shell_output("#{bin}/bnpm --help")

    (testpath/"package.json").write <<~JSON
      {
        "name": "test-pkg",
        "version": "0.0.0",
        "description": "A test package"
      }
    JSON

    output = shell_output("#{bin}/bnpm install 2>&1 || true")
    assert_match(/No dependencies|Done/i, output)
  end
end
