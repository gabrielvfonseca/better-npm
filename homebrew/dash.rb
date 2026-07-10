# typed: true
# frozen_string_literal: true

# Dash — JavaScript package manager
# This file can be used as a standalone formula for `brew install` from URL.
# Usage: brew install https://raw.githubusercontent.com/org/dash/main/homebrew/dash.rb

class Dash < Formula
  desc "JavaScript package manager — fast, secure, and reliable"
  homepage "https://github.com/org/dash"
  license "MIT"
  head "https://github.com/org/dash.git", branch: "main"

  stable do
    url "https://github.com/org/dash/archive/refs/tags/v0.0.0.tar.gz"
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
      bin/"dash", "bash",
      shell_function: :bash
    ).output

    zsh_completion.install Utils::ShellCompletions.new(
      bin/"dash", "zsh",
      shell_function: :zsh
    ).output

    fish_completion.install Utils::ShellCompletions.new(
      bin/"dash", "fish",
      shell_function: :fish
    ).output
  end

  def generate_man_pages
    system bin/"dash", "man", "--output", buildpath/"man"
    man1.install Dir[buildpath/"man/**/*.1"]
  end

  def post_install
    (var/"dash/cache").mkpath
    (var/"dash/data").mkpath
  end

  def caveats
    <<~EOS
      Dash is installed as `dash`.

      Cache: #{var}/dash/cache
      Data:  #{var}/dash/data

      Quick start:
        dash help
        dash init my-project
        cd my-project && dash install
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/dash --version")
    assert_match "Usage:", shell_output("#{bin}/dash --help")

    (testpath/"package.json").write <<~JSON
      {
        "name": "test-pkg",
        "version": "0.0.0",
        "description": "A test package"
      }
    JSON

    output = shell_output("#{bin}/dash install 2>&1 || true")
    assert_match(/No dependencies|Done/i, output)
  end
end
