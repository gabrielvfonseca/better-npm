# Dash

A comprehensive package management platform — registry, CLI, SDK, and tooling for the modern ecosystem.

## Structure

```
dash/
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

- **Bun** >= 1.2.0
- **Rust** >= 1.84.0
- **Docker** (for services)
- **Buf** (for protobuf)

## Quick Start

```bash
# Install dependencies
bun install

# Start development environment
bun run dev

# Build all packages and apps
bun run build

# Run all tests
bun run test

# Lint and typecheck
bun run lint
bun run typecheck
```

## Key Scripts

| Command | Description |
|---|---|
| `bun run dev` | Start all services in dev mode |
| `bun run build` | Build all packages and apps |
| `bun run lint` | Lint all TS/JS code with Biome |
| `bun run typecheck` | TypeScript type checking |
| `bun run test` | Run all tests |
| `bun run changeset` | Create a changeset for versioning |
| `bun run db:migrate` | Run database migrations |
| `bun run proto:gen` | Generate code from protobuf definitions |
| `cargo build` | Build all Rust crates |
| `cargo test` | Test all Rust crates |

## CI/CD

- **CI**: Lint, typecheck, test, and build on every PR
- **Release**: Automated via Changesets on merge to `main`
- **Docker**: Multi-stage builds with Bake

## License

MIT
