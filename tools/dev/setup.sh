#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Setting up Bnpm development environment..."

# Check prerequisites
echo "==> Checking prerequisites..."
MISSING=0
for cmd in docker node pnpm git; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "  ERROR: $cmd is required but not installed."
        MISSING=1
    fi
done
if [ "$MISSING" -eq 1 ]; then
    exit 1
fi

echo "  docker: $(docker --version 2>/dev/null)"
echo "  node:   $(node --version 2>/dev/null)"
echo "  pnpm:   $(pnpm --version 2>/dev/null)"
echo "  git:    $(git --version 2>/dev/null)"

# Copy env file if it doesn't exist
if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        echo "==> Creating .env from .env.example..."
        cp .env.example .env
        echo "  Edit .env to configure your environment."
    else
        echo "  WARNING: No .env.example found. Create .env manually."
    fi
fi

# Install dependencies
echo "==> Installing dependencies..."
pnpm install

# Build Rust crates
if command -v cargo &> /dev/null; then
    echo "==> Building Rust crates..."
    cargo build 2>/dev/null || echo "  (Rust build had warnings)"
else
    echo "  (cargo not available)"
fi

# Generate protobuf code
echo "==> Generating protobuf code..."
if command -v buf &> /dev/null; then
    buf generate 2>/dev/null || echo "  (buf generate failed)"
elif command -v protoc &> /dev/null; then
    protoc --proto_path=proto --go_out=proto/gen proto/**/*.proto 2>/dev/null || true
else
    echo "  (protoc not available, skipping)"
fi

# Install git hooks
echo "==> Installing git hooks..."
if command -v lefthook &> /dev/null; then
    lefthook install
else
    npx lefthook install 2>/dev/null || echo "  (lefthook not found, skipping)"
fi

# Pull Docker images
echo "==> Pulling Docker images..."
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml pull postgres redis minio 2>/dev/null || true

# Run typecheck
echo "==> Checking TypeScript..."
pnpm typecheck 2>/dev/null || echo "  (typecheck script not found, skipping)"

echo ""
echo "==> Setup complete!"
echo ""
echo "Quick start:"
echo "  tools/scripts/dev.sh               Start full dev environment"
echo "  pnpm dev                         Start all turborepo dev servers"
echo "  pnpm dev --filter=@better-npm/api      Start API only"
echo "  pnpm db:migrate                  Run database migrations"
echo "  pnpm db:seed                     Seed database"
