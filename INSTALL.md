# Installing Bnpm

Bnpm is a JavaScript package manager written in TypeScript and Rust. These
instructions will help you install Bnpm on your system.

## Prerequisites

Before installing Bnpm, ensure you have the following prerequisites:

- **Node.js** 22+ and **pnpm** 10+ — [Install pnpm](https://pnpm.io)
- **Rust** 1.80+ (edition 2024) — [Install Rust](https://rustup.rs)
- **Docker** (optional, for local infrastructure) — [Install Docker](https://docker.com)
- **Buf CLI** (optional, for protobuf codegen) — `brew install buf`

## Quick Install

### macOS (Homebrew)

```bash
brew tap bnpm-io/bnpm
brew install bnpm
```

### Linux (APT)

```bash
# Add the Bnpm repository
echo "deb [trusted=yes] https://apt.bnpm.dev/ /" | sudo tee /etc/apt/sources.list.d/bnpm.list
sudo apt update
sudo apt install bnpm
```

### Linux (apk — Alpine)

```bash
apk add bnpm
```

### Windows (Scoop)

```powershell
scoop bucket add bnpm https://github.com/bnpm-io/scoop-bucket.git
scoop install bnpm
```

### npm / pnpm (global)

```bash
npm install -g @better-npm/cli
# or
pnpm add -g @better-npm/cli
```

### Cargo (from source)

```bash
cargo install bnpm-cli
```

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/bnpm-io/bnpm.git
cd bnpm
```

### Install Dependencies

```bash
pnpm install
```

### Build All Packages

```bash
pnpm build
```

### Build the CLI Only

```bash
cargo build --release -p bnpm-cli
```

The binary will be at `target/release/bnpm`.

### Install the Built Binary

```bash
# Add to your PATH (recommended)
cp target/release/bnpm ~/.local/bin/

# Or install system-wide (macOS/Linux)
sudo cp target/release/bnpm /usr/local/bin/
```

## Platform-Specific Notes

### macOS (Apple Silicon)

Bnpm runs natively on Apple Silicon. No Rosetta or emulation is required. The
Homebrew formula ships an `arm64` binary.

### Linux

Bnpm is tested on:
- Ubuntu 22.04+ / Debian 12+
- Fedora 38+
- Arch Linux
- Alpine 3.19+

Pre-built binaries are available for `x86_64` and `aarch64`.

### Windows (WSL2)

Bnpm does not currently support native Windows. Use WSL2 with an Ubuntu
distribution:

1. Install [WSL2](https://learn.microsoft.com/en-us/windows/wsl/install)
2. Launch Ubuntu from the Start menu
3. Install Bnpm via the Linux instructions above

## Verifying Installation

```bash
# Check the installed version
bnpm --version

# Display help
bnpm --help

# Run the self-check
bnpm doctor
```

You should see output similar to:

```
$ bnpm --version
bnpm 0.1.0 (aarch64-apple-darwin)
```

## Next Steps

- **[Quick Start](https://bnpm.dev/docs/quick-start)** — Create your first
  project with Bnpm
- **[Configuration](https://bnpm.dev/docs/configuration)** — Learn how to
  configure Bnpm for your project
- **[CLI Reference](https://bnpm.dev/docs/cli)** — Complete command reference
- **[Contributing](CONTRIBUTING.md)** — Learn how to contribute to Bnpm

## Troubleshooting

### Permission Denied

If you get a permission error when running `bnpm`, ensure the binary is
executable:

```bash
chmod +x $(which bnpm)
```

### Command Not Found

Ensure Bnpm is in your PATH. If you installed via Homebrew, run:

```bash
brew link bnpm
```

### Build Errors

If you encounter build errors when building from source:

1. Ensure Rust is up to date: `rustup update`
2. Ensure pnpm is up to date: `pnpm upgrade`
3. Clean and rebuild: `cargo clean && pnpm build`

For further help, see [SUPPORT.md](SUPPORT.md).
