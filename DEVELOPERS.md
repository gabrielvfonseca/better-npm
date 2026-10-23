# Developer Guide

This guide covers everything you need to know to develop Bnpm effectively.

## Project Overview

Bnpm is a modern JavaScript package manager built with a hybrid architecture:

- **Rust** — Core engine (parsing, resolution, installation)
- **TypeScript** — CLI interface, API server, web dashboard

The monorepo uses [Turborepo](https://turbo.build/repo) for build orchestration,
[Changesets](https://github.com/changesets/changesets) for versioning, and
[Biome](https://biomejs.dev/) for TypeScript tooling.

## Architecture

```
┌─────────────────────────────┐
│         CLI (bnpm)          │
│  TypeScript / Rust (bnpm-cli)│
├─────────────────────────────┤
│         API Server          │
│     Hono (apps/api/)        │
├─────────────────────────────┤
│        Web ⟦Bnpmboard⟧        │
│    Next.js (apps/web/)      │
├─────────────────────────────┤
│     Documentation Site      │
│    apps/docs/               │
├─────────────────────────────┤
│       Rust Workspace        │
│  core / parser / registry   │
│  installer (crates/*)       │
├─────────────────────────────┤
│     TypeScript Packages     │
│  cli / config / types       │
│  (packages/*)               │
└─────────────────────────────┘
```

## Setting Up the Development Environment

### Step 1: Install System Dependencies

```bash
# macOS
brew install just buf

# Linux (Ubuntu/Debian)
sudo apt install just protobuf-compiler
```

### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### Step 3: Install pnpm

```bash
npm install -g pnpm@10
```

### Step 4: Clone and Install

```bash
git clone https://github.com/bnpm-io/bnpm.git
cd bnpm
pnpm install
```

### Step 5: Build Everything

```bash
pnpm run build
```

## Running the Full Stack Locally

### Infrastructure (Docker)

Start local infrastructure (database, cache, registry):

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

### Services

In separate terminals:

```bash
# API server (port 3000)
pnpm dev --filter @better-npm/api

# Web dashboard (port 3001)
pnpm dev --filter @better-npm/web

# Docs site (port 3002)
pnpm dev --filter @better-npm/docs
```

### CLI from Source

```bash
# Run the CLI directly
cargo run -- -p bnpm-cli -- help

# Or if compiled
cargo build --release -p bnpm-cli
./target/release/bnpm help
```

## Monorepo Layout

```
bnpm/
├── apps/               # Application services
│   ├── api/            # Hono API server
│   ├── cli/            # CLI binary entrypoint
│   ├── docs/           # Documentation site
│   └── web/            # Next.js web app
├── crates/             # Rust crates
│   ├── core/           # Core library
│   ├── installer/      # Package installer
│   ├── parser/         # Manifest parser
│   └── registry/       # Package registry client
├── packages/           # TypeScript packages (npm)
│   ├── cli/            # CLI library
│   ├── config/         # Configuration utilities
│   └── types/          # Shared types
├── proto/              # Protobuf definitions
├── tools/              # Build and dev tools
│   └── generators/     # Plop code generators
├── turbo.json          # Turborepo config
├── package.json        # Workspace root
├── Cargo.toml          # Rust workspace
└── tsconfig.json       # TS base config
```

## Development Commands Reference

| Command | Description |
|---------|-------------|
| `pnpm build` | Build all TypeScript packages |
| `pnpm dev` | Run all apps in development mode |
| `pnpm test` | Run all TypeScript tests |
| `pnpm lint` | Lint TypeScript with Biome |
| `pnpm typecheck` | TypeScript type checking |
| `pnpm clean` | Clean all build artifacts |
| `pnpm gen` | Run code generators (Plop) |
| `cargo build --workspace` | Build all Rust crates |
| `cargo test --workspace` | Run all Rust tests |
| `cargo clippy --workspace` | Lint all Rust crates |
| `cargo fmt` | Format all Rust code |

## Database Setup and Migrations

Bnpm uses SQLite for local state and PostgreSQL for the registry API.

### Local SQLite

The SQLite database is created automatically at `~/.bnpm/bnpm.db`.

### PostgreSQL (Development)

```bash
# Start PostgreSQL
docker compose -f docker/docker-compose.dev.yml up -d postgres

# Run migrations
pnpm db:migrate --filter @better-npm/api

# Seed development data
pnpm db:seed --filter @better-npm/api
```

### Migration Files

Migrations are located in `apps/api/src/db/migrations/` and use
[Drizzle ORM](https://orm.drizzle.team/).

```bash
# Generate a new migration
pnpm db:generate --filter @better-npm/api

# Apply migrations
pnpm db:migrate --filter @better-npm/api
```

## Working with Protobuf

Protobuf definitions are in `proto/`. We use [Buf](https://buf.build) for code
generation.

### Prerequisites

```bash
brew install buf
```

### Generate Code

```bash
pnpm gen:proto
```

### Adding a New Protobuf Definition

1. Create the `.proto` file in `proto/`.
2. Run `pnpm gen:proto` to generate Rust and TS types.
3. Import and use the generated types.

### Linting Protobuf

```bash
buf lint
```

## Building and Testing Rust Crates

### Build

```bash
# Build all crates
cargo build --workspace

# Build a specific crate (release)
cargo build --release -p bnpm-core

# Check compilation (faster)
cargo check --workspace
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p bnpm-parser

# Run a specific test
cargo test -p bnpm-parser -- --test-threads=1 test_name

# Run tests with logging
cargo test -- --nocapture
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Building and Testing TypeScript Packages

### Build

```bash
# Build all packages
pnpm build

# Build a specific package
pnpm build --filter @better-npm/cli

# Build in watch mode
pnpm build --watch
```

### Test

```bash
# Run all tests
pnpm test

# Run tests in a specific package
pnpm test --filter @better-npm/cli

# Run tests with coverage
pnpm test --coverage

# Run tests in watch mode
pnpm test --watch
```

### Lint

```bash
# Check linting
pnpm lint

# Auto-fix
pnpm lint --fix
```

## Debugging Tips

### TypeScript Debugging

1. Set `DEBUG=1` environment variable for verbose logging:
   ```bash
   DEBUG=1 pnpm dev --filter @better-npm/api
   ```
2. Use Chrome DevTools for Node.js debugging:
   ```bash
   pnpm --inspect-brk run dev --filter @better-npm/api
   ```
3. VS Code launch configurations are in `.vscode/launch.json`.

### Rust Debugging

1. Build with debug symbols (default for `cargo build`).
2. Use `RUST_LOG=debug` for detailed logs:
   ```bash
   RUST_LOG=debug cargo run -p bnpm-cli
   ```
3. Use `lldb` or `rust-gdb` for step-through debugging:
   ```bash
   rust-lldb target/debug/bnpm
   ```
4. Use `tracing` spans for performance breakdowns.

### Common Issues

- **Port already in use**: Kill the existing process or change the port in `.env`.
- **Protobuf codegen stale**: Run `pnpm gen:proto` after updating `.proto` files.
- **Rust compilation slow**: Use `cargo check` instead of `cargo build` for faster
  feedback cycles.

## Performance Profiling

### TypeScript Profiling

```bash
# CPU profiling with Node.js
node --cpu-prof --heap-prof node_modules/.bin/tsx apps/api/src/index.ts

# Memory usage
pnpm test --coverage --max-old-space-size=4096
```

### Rust Profiling

```bash
# Build with profiling symbols
RUSTFLAGS="-g" cargo build --release -p bnpm-core

# Use perf (Linux)
perf record ./target/release/bnpm install
perf report

# Use samply (macOS/Linux)
cargo install samply
samply record ./target/release/bnpm install
```

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `BNPM_HOME` | `~/.bnpm` | Data directory |
| `BNPM_REGISTRY` | `https://registry.bnpm.dev` | Registry URL |
| `BNPM_CACHE_DIR` | `$BNPM_HOME/cache` | Cache directory |
| `BNPM_LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |
| `BNPM_NO_COLOR` | `false` | Disable colored output |
| `DATABASE_URL` | — | PostgreSQL connection string |
| `REDIS_URL` | — | Redis connection string |
| `PORT` | `3000` | API server port |
| `NEXT_PUBLIC_API_URL` | `http://localhost:3000` | Public API URL |

### .env File

Copy the template:

```bash
cp .env.example .env
```

Then edit `.env` with your local configuration.
