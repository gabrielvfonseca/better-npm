You are working on the Dash monorepo — a JavaScript package manager written in TypeScript and Rust.

## Key Commands

- `bun run build` — Build all TS packages
- `cargo build --workspace` — Build all Rust crates
- `bun run test` — Run all TS tests
- `cargo test --workspace` — Run all Rust tests
- `bun run lint` — Lint TS with Biome
- `cargo clippy --workspace` — Lint Rust
- `bun run gen` — Run code generators (Plop)

## Architecture

The CLI binary is `dash-cli` (crates/cli/), built as `dash`. Packages in packages/ are published to npm. Crates in crates/ are internal workspace members.

## Important Paths

- Root Cargo.toml — Rust workspace with all shared dependencies
- Root tsconfig.json — TypeScript base config with path aliases
- turbo.json — Build task orchestration
- tools/generators/plopfile.js — Code generators with Handlebars templates

When writing code, follow existing patterns. Never add comments unless the code's purpose is unclear. Never add empty export stubs.
