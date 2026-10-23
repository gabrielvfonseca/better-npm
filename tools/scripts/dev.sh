#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Starting Bnpm development environment..."

# Check prerequisites
command -v docker &> /dev/null || { echo "docker is required"; exit 1; }
command -v node &> /dev/null || { echo "node is required"; exit 1; }

# Start infrastructure
echo "==> Starting infrastructure (Postgres, Redis, Minio)..."
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml up -d postgres redis minio

# Wait for services to be ready
echo "==> Waiting for services..."
for i in $(seq 1 30); do
    if docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml exec postgres pg_isready -U bnpm 2>/dev/null; then
        echo "  Postgres is ready."
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "  WARNING: Postgres did not become ready in time."
    fi
    sleep 1
done

# Run database migrations
echo "==> Running migrations..."
if command -v pnpm &> /dev/null; then
    pnpm db:migrate
elif command -v npx &> /dev/null; then
    npx turbo db:migrate
else
    echo "  (no package manager available, skipping migrations)"
fi

# Seed database if requested
if [ "${BNPM_SEED:-0}" = "1" ]; then
    echo "==> Seeding database..."
    psql "${DATABASE_URL:-postgresql://bnpm:bnpm_dev_password@localhost:5432/bnpm_dev}" -f database/seeds/seed.sql 2>/dev/null || echo "  (seeding skipped)"
fi

# Start dev servers
echo "==> Starting development servers..."
if command -v pnpm &> /dev/null; then
    pnpm dev
elif command -v npx &> /dev/null && command -v turbo &> /dev/null; then
    npx turbo dev --parallel
else
    echo "WARNING: turbo not found. Run individual apps manually."
    echo "  pnpm dev --filter=@better-npm/api"
    echo "  pnpm dev --filter=@better-npm/registry"
    echo "  pnpm dev --filter=@better-npm/website"
fi

echo "==> Development environment ready."
