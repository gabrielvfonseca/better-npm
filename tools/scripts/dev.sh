#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Starting Dash development environment..."

# Check prerequisites
command -v docker &> /dev/null || { echo "docker is required"; exit 1; }
command -v node &> /dev/null || { echo "node is required"; exit 1; }

# Start infrastructure
echo "==> Starting infrastructure (Postgres, Redis, Minio)..."
docker compose -f docker-compose.yml -f tools/dev/docker-compose.dev.yml up -d postgres redis minio

# Wait for services to be ready
echo "==> Waiting for services..."
for i in $(seq 1 30); do
    if docker compose -f docker-compose.yml -f tools/dev/docker-compose.dev.yml exec postgres pg_isready -U dash 2>/dev/null; then
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
if command -v bun &> /dev/null; then
    bun run db:migrate
elif command -v npx &> /dev/null; then
    npx turbo db:migrate
else
    echo "  (no package manager available, skipping migrations)"
fi

# Seed database if requested
if [ "${DASH_SEED:-0}" = "1" ]; then
    echo "==> Seeding database..."
    psql "${DATABASE_URL:-postgresql://dash:dash_dev_password@localhost:5432/dash_dev}" -f database/seeds/seed.sql 2>/dev/null || echo "  (seeding skipped)"
fi

# Start dev servers
echo "==> Starting development servers..."
if command -v bun &> /dev/null; then
    bun run dev
elif command -v npx &> /dev/null && command -v turbo &> /dev/null; then
    npx turbo dev --parallel
else
    echo "WARNING: turbo not found. Run individual apps manually."
    echo "  bun run dev --filter=@dash/api"
    echo "  bun run dev --filter=@dash/registry"
    echo "  bun run dev --filter=@dash/website"
fi

echo "==> Development environment ready."
