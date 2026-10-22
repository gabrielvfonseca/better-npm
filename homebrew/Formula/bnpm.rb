class Bnpm < Formula
  desc "JavaScript package manager — fast, secure, and reliable"
  homepage "https://github.com/org/bnpm"
  license "MIT"
  version "0.0.0"

  stable do
    url "https://github.com/org/bnpm/archive/refs/tags/v#{version}.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"

    resource "cargo-deps" do
      url "https://github.com/org/bnpm/releases/download/v#{version}/bnpm-v#{version}-cargo-deps.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  head do
    url "https://github.com/org/bnpm.git", branch: "main"

    depends_on "rust" => :build
  end

  bottle do
    root_url "https://github.com/org/bnpm/releases/download/v#{version}"
    rebuild 0

    sha256 cellar: :any_skip_relocation, arm64_sequoia:  "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, arm64_sonoma:   "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, arm64_ventura:  "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, x86_64_linux:   "0000000000000000000000000000000000000000000000000000000000000000"
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
    # Set OpenSSL environment for Rust build
    ENV["OPENSSL_NO_VENDOR"] = "1"
    ENV["OPENSSL_DIR"] = Formula["openssl@3"].opt_prefix

    if build.stable?
      resource("cargo-deps").stage do
        (buildpath/"vendor").install Dir["*"]
      end
      ENV["CARGO_HOME"] = buildpath/".cargo"
      ENV.prepend_path "PATH", buildpath/".cargo/bin"
    end

    # Build the CLI release binary with all features
    system "cargo", "build",
           "--release",
           "--package", "bnpm-cli",
           "--bin", "bnpm",
           "--features", "cli/native-tls"

    # Install the binary
    bin.install "target/release/bnpm"

    # Generate shell completions
    generate_completions

    # Install man pages
    generate_man_pages

    # Install license
    ln_sf buildpath/"LICENSE", prefix/"LICENSE"

    # Install the config file template
    etc.install "crates/cli/config.default.toml" => "bnpm.toml" if File.exist? "crates/cli/config.default.toml"
  end

  def post_install
    # Create the cache and data directories
    (var/"bnpm/cache").mkpath
    (var/"bnpm/data").mkpath
  end

  def generate_completions
    # Generate bash completions
    system bin/"bnpm", "completions", "--shell", "bash",
           "--output", buildpath/"completions/bnpm.bash"
    bash_completion.install "completions/bnpm.bash"

    # Generate zsh completions
    system bin/"bnpm", "completions", "--shell", "zsh",
           "--output", buildpath/"completions/_bnpm"
    zsh_completion.install "completions/_bnpm"

    # Generate fish completions
    system bin/"bnpm", "completions", "--shell", "fish",
           "--output", buildpath/"completions/bnpm.fish"
    fish_completion.install "completions/bnpm.fish"
  end

  def generate_man_pages
    # Generate man pages from the CLI's help
    system bin/"bnpm", "man", "--output", buildpath/"man/man1"
    man1.install Dir[buildpath/"man/man1/*.1"]
  end

  def caveats
    <<~EOS
      Bnpm is installed as `bnpm`.

      Shell completions have been installed to:
        #{bash_completion}
        #{zsh_completion}
        #{fish_completion}

      Cache directory: #{var}/bnpm/cache
      Data directory:  #{var}/bnpm/data
      Config file:     #{etc}/bnpm.toml

      To get started:
        bnpm help
        bnpm init
        bnpm install <package>
    EOS
  end

  test do
    # Test version output
    assert_match "bnpm #{version}", shell_output("#{bin}/bnpm --version")

    # Test help output
    assert_match "Usage: bnpm", shell_output("#{bin}/bnpm --help")

    # Test init creates a package.json
    mkdir testpath/"test-project"
    cd testpath/"test-project" do
      system bin/"bnpm", "init", "--name", "test-pkg", "--yes"
      assert_path_exists testpath/"test-project/package.json"

      json = JSON.parse((testpath/"test-project/package.json").read)
      assert_equal "test-pkg", json["name"]
      assert_equal version.to_s, json["version"]
    end

    # Test cache operations
    system bin/"bnpm", "cache", "stats"
    system bin/"bnpm", "cache", "clear"

    # Test search (requires network, may fail in offline CI)
    unless ENV["HOMEBREW_TEST_OFFLINE"]
      output = shell_output("#{bin}/bnpm search lodash 2>&1 || true")
      assert_match(/lodash|No results/i, output)
    end
  end
end
