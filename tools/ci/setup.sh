#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Setting up CI environment..."

# Install system dependencies (Linux/Debian based)
if [ -f /etc/os-release ] && grep -qi debian /etc/os-release 2>/dev/null; then
    echo "==> Installing system dependencies..."
    sudo apt-get update -qq
    sudo apt-get install -y -qq --no-install-recommends \
        ca-certificates \
        curl \
        gnupg \
        build-essential \
        pkg-config \
        libssl-dev \
        postgresql-client \
        redis-tools \
        protobuf-compiler \
        jq
fi

# Setup Node.js via nvm
if [ -n "${NVM_DIR:-}" ] && [ -f "$NVM_DIR/nvm.sh" ]; then
    . "$NVM_DIR/nvm.sh"
    nvm install "${NODE_VERSION:-22}" --no-progress
    nvm use "${NODE_VERSION:-22}"
elif command -v nvm &> /dev/null; then
    nvm install "${NODE_VERSION:-22}"
    nvm use "${NODE_VERSION:-22}"
fi

# Install bun if not available
if ! command -v bun &> /dev/null; then
    echo "==> Installing bun..."
    curl -fsSL https://bun.sh/install | bash
    export PATH="$HOME/.bun/bin:$PATH"
fi

# Install turbo globally if not available
if ! command -v turbo &> /dev/null; then
    echo "==> Installing turbo..."
    npm install -g turbo
fi

# Install dependencies
echo "==> Installing dependencies..."
bun install --frozen-lockfile 2>/dev/null || npm ci

# Fetch Rust dependencies
if command -v cargo &> /dev/null; then
    echo "==> Fetching Rust dependencies..."
    cargo fetch 2>/dev/null || true
fi

# Generate protobuf code
if command -v buf &> /dev/null; then
    echo "==> Generating protobuf code..."
    buf generate 2>/dev/null || true
fi

# Verify toolchain
echo "==> Verifying toolchain..."
echo "  node:   $(node --version 2>/dev/null || echo 'N/A')"
echo "  npm:    $(npm --version 2>/dev/null || echo 'N/A')"
echo "  bun:    $(bun --version 2>/dev/null || echo 'N/A')"
echo "  turbo:  $(turbo --version 2>/dev/null || echo 'N/A')"
echo "  cargo:  $(cargo --version 2>/dev/null || echo 'N/A')"
echo "  rustc:  $(rustc --version 2>/dev/null || echo 'N/A')"

echo ""
echo "==> CI environment ready."
