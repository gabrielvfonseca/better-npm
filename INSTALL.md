# Installing Dash

Dash is a JavaScript package manager written in TypeScript and Rust. These
instructions will help you install Dash on your system.

## Prerequisites

Before installing Dash, ensure you have the following prerequisites:

- **Bun** 1.1+ — [Install Bun](https://bun.sh)
- **Rust** 1.80+ (edition 2024) — [Install Rust](https://rustup.rs)
- **Docker** (optional, for local infrastructure) — [Install Docker](https://docker.com)
- **Buf CLI** (optional, for protobuf codegen) — `brew install buf`

## Quick Install

### macOS (Homebrew)

```bash
brew tap dashapp/dash
brew install dash
```

### Linux (APT)

```bash
# Add the Dash repository
echo "deb [trusted=yes] https://apt.dash.dev/ /" | sudo tee /etc/apt/sources.list.d/dash.list
sudo apt update
sudo apt install dash
```

### Linux (apk — Alpine)

```bash
apk add dash
```

### Windows (Scoop)

```powershell
scoop bucket add dash https://github.com/dashapp/scoop-bucket.git
scoop install dash
```

### npm / pnpm / bun (global)

```bash
npm install -g @dash/cli
# or
pnpm add -g @dash/cli
# or
bun install -g @dash/cli
```

### Cargo (from source)

```bash
cargo install dash-cli
```

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/dashapp/dash.git
cd dash
```

### Install Dependencies

```bash
bun install
```

### Build All Packages

```bash
bun run build
```

### Build the CLI Only

```bash
cargo build --release -p dash-cli
```

The binary will be at `target/release/dash`.

### Install the Built Binary

```bash
# Add to your PATH (recommended)
cp target/release/dash ~/.local/bin/

# Or install system-wide (macOS/Linux)
sudo cp target/release/dash /usr/local/bin/
```

## Platform-Specific Notes

### macOS (Apple Silicon)

Dash runs natively on Apple Silicon. No Rosetta or emulation is required. The
Homebrew formula ships an `arm64` binary.

### Linux

Dash is tested on:
- Ubuntu 22.04+ / Debian 12+
- Fedora 38+
- Arch Linux
- Alpine 3.19+

Pre-built binaries are available for `x86_64` and `aarch64`.

### Windows (WSL2)

Dash does not currently support native Windows. Use WSL2 with an Ubuntu
distribution:

1. Install [WSL2](https://learn.microsoft.com/en-us/windows/wsl/install)
2. Launch Ubuntu from the Start menu
3. Install Dash via the Linux instructions above

## Verifying Installation

```bash
# Check the installed version
dash --version

# Display help
dash --help

# Run the self-check
dash doctor
```

You should see output similar to:

```
$ dash --version
dash 0.1.0 (aarch64-apple-darwin)
```

## Next Steps

- **[Quick Start](https://dash.dev/docs/quick-start)** — Create your first
  project with Dash
- **[Configuration](https://dash.dev/docs/configuration)** — Learn how to
  configure Dash for your project
- **[CLI Reference](https://dash.dev/docs/cli)** — Complete command reference
- **[Contributing](CONTRIBUTING.md)** — Learn how to contribute to Dash

## Troubleshooting

### Permission Denied

If you get a permission error when running `dash`, ensure the binary is
executable:

```bash
chmod +x $(which dash)
```

### Command Not Found

Ensure Dash is in your PATH. If you installed via Homebrew, run:

```bash
brew link dash
```

### Build Errors

If you encounter build errors when building from source:

1. Ensure Rust is up to date: `rustup update`
2. Ensure Bun is up to date: `bun upgrade`
3. Clean and rebuild: `cargo clean && bun run build`

For further help, see [SUPPORT.md](SUPPORT.md).
