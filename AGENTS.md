# Bnpm Monorepo — AI Agent Instructions

## Repository Structure

- **TypeScript packages** in `packages/*` — build with `tsup`, lint with Biome
- **Rust crates** in `crates/*` — edition 2024, clippy + rustfmt
- **Application services** in `apps/*` — Hono for API, Next.js for web
- **Protobuf definitions** in `proto/*` — buf for codegen
- **Turborepo** for build orchestration
- **Changesets** for versioning

## Conventions

- **TypeScript**: strict mode, no unused variables, path aliases via tsconfig
- **Rust**: clippy::pedantic, use workspace inheritance, avoid unsafe
- **Commits**: conventional commits (`feat:`, `fix:`, `chore:`, etc.)
- **PRs**: require semver label, conventional title
- **Tests**: vitest for TS, cargo test for Rust
- **Formatting**: biome for TS/JS, rustfmt for Rust

## Commands

| Command | Description |
|---------|-------------|
| `bun run build` | Build all TS packages |
| `bun run test` | Run all TS tests |
| `bun run lint` | Lint TS with Biome |
| `bun run typecheck` | TS type checking |
| `cargo build --workspace` | Build all Rust crates |
| `cargo test --workspace` | Run all Rust tests |
| `cargo clippy --workspace` | Lint Rust |
| `bun run gen` | Run code generators (Plop) |

## Best Practices

- Never add comments unless the code's purpose is unclear.
- Never add empty export stubs.
- Follow existing patterns in the codebase.
- Keep functions small and focused.
- Use meaningful variable and function names.
- Prefer named exports over default exports.
- Use `type` imports for type-only imports.

## Important Paths

- `Root Cargo.toml` — Rust workspace with all shared dependencies
- `Root tsconfig.json` — TypeScript base config with path aliases
- `turbo.json` — Build task orchestration
- `tools/generators/plopfile.js` — Code generators with Handlebars templates
