#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Cleaning Dash monorepo..."

# Remove build artifacts
echo "==> Removing build artifacts..."
rm -rf apps/*/dist apps/*/.next apps/*/build apps/*/.output apps/*/storybook-static
rm -rf packages/*/dist packages/*/lib packages/*/build packages/*/.next
rm -rf crates/*/target
rm -rf node_modules .turbo

# Remove coverage reports
echo "==> Removing coverage reports..."
rm -rf coverage .nyc_output apps/*/coverage packages/*/coverage

# Remove generated proto code
echo "==> Removing generated code..."
rm -rf proto/gen proto/generated proto/out

# Remove logs
echo "==> Removing logs..."
rm -rf logs *.log apps/*/*.log packages/*/*.log

# Remove temp files
echo "==> Removing temp files..."
rm -rf /tmp/dash-* .tmp

# Remove Docker volumes (optional, skip with SKIP_VOLUMES=1)
if [ -z "${SKIP_VOLUMES:-}" ]; then
    echo "==> Removing Docker volumes..."
    docker compose -f docker-compose.yml -f tools/dev/docker-compose.dev.yml down -v 2>/dev/null || true
fi

# Clean Rust artifacts
if command -v cargo &> /dev/null; then
    echo "==> Cleaning Rust artifacts..."
    cargo clean 2>/dev/null || true
fi

echo "==> Clean complete."
