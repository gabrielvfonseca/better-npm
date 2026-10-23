# Better-npm (experimental)

A comprehensive package management platform — registry, CLI, SDK, and tooling for the modern ecosystem.

## Structure

```
better-npm/
├── apps/          # Application services (API, registry, website, workers, etc.)
├── crates/        # Rust crates (CLI, resolver, installer, graph, crypto, etc.)
├── packages/      # TypeScript packages (SDK, client, UI, schemas, configs, etc.)
├── tools/         # Build, CI, Docker, profiling, and scaffolding tools
├── proto/         # Protobuf definitions (registry, auth, telemetry, etc.)
├── database/      # Migrations, seeds, and schema definitions
├── examples/      # Usage examples across frameworks
├── docs/          # Documentation
└── benchmarks/    # Benchmarking methodology and results
```

## Prerequisites

- **Node.js** >= 22 and **pnpm** >= 10
- **Rust** >= 1.84.0
- **Docker** (for services)
- **Buf** (for protobuf)

## Quick Start

```bash
# Install dependencies
pnpm install

# Start development environment
pnpm dev

# Build all packages and apps
pnpm build

# Run all tests
pnpm test

# Lint and typecheck
pnpm lint
pnpm typecheck
```

## Key Scripts

| Command | Description |
|---|---|
| `pnpm dev` | Start all services in dev mode |
| `pnpm build` | Build all packages and apps |
| `pnpm lint` | Lint all TS/JS code with Biome |
| `pnpm typecheck` | TypeScript type checking |
| `pnpm test` | Run all tests |
| `pnpm changeset` | Create a changeset for versioning |
| `pnpm db:migrate` | Run database migrations |
| `pnpm proto:gen` | Generate code from protobuf definitions |
| `cargo build` | Build all Rust crates |
| `cargo test` | Test all Rust crates |

## CI/CD

- **CI**: Lint, typecheck, test, and build on every PR
- **Release**: Automated via Changesets on merge to `main`
- **Docker**: Multi-stage builds with Bake

## License

MIT
