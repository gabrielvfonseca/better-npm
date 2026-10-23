#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Initializing Bnpm monorepo..."

# Create required directories
echo "==> Creating directory structure..."
mkdir -p proto/registry/v1 proto/packages/v1 proto/search/v1 proto/auth/v1 proto/telemetry/v1 proto/common/v1
mkdir -p database/migrations database/seeds database/schema
mkdir -p docker tools/scripts tools/generators/package/templates tools/release tools/dev tools/ci tools/benchmarks tools/profiling tools/fixtures/test-packages tools/templates/new-app
mkdir -p examples/react/src examples/vue/src examples/svelte/src examples/node/src examples/hono/src examples/nextjs/app examples/library/src examples/monorepo
mkdir -p docs benchmarks

# Install dependencies
if command -v pnpm &> /dev/null; then
    echo "==> Installing dependencies with pnpm..."
    pnpm install
elif command -v npm &> /dev/null; then
    echo "==> Installing dependencies with npm..."
    npm install
else
    echo "WARNING: No package manager found. Install pnpm or npm."
fi

# Build Rust crates
if command -v cargo &> /dev/null; then
    echo "==> Building Rust crates..."
    cargo build 2>/dev/null || echo "  (Rust build skipped)"
else
    echo "  (cargo not available, skipping Rust build)"
fi

# Generate protobuf code
if command -v buf &> /dev/null; then
    echo "==> Generating protobuf code..."
    buf generate
elif command -v protoc &> /dev/null; then
    echo "==> Generating protobuf code with protoc..."
    protoc --proto_path=proto --go_out=proto/gen proto/**/*.proto 2>/dev/null || true
else
    echo "  (buf/protoc not available, skipping protobuf generation)"
fi

# Setup git hooks
if command -v lefthook &> /dev/null; then
    echo "==> Installing git hooks..."
    lefthook install
elif command -v npx &> /dev/null; then
    npx lefthook install 2>/dev/null || true
fi

# Initialize database
if command -v docker &> /dev/null; then
    echo "==> Starting development database..."
    docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml up -d postgres redis 2>/dev/null || echo "  (Docker compose failed, skipping)"
fi

echo "==> Done! Bnpm monorepo initialized."
echo ""
echo "Next steps:"
echo "  tools/scripts/dev.sh    Start development environment"
echo "  pnpm run dev             Start turborepo dev servers"
