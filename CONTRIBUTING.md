# Contributing to Bnpm

Thank you for your interest in contributing to Bnpm! We welcome contributions
from everyone. By participating in this project, you agree to abide by our
[Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Fork and Clone

1. Fork the repository on GitHub.
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/bnpm.git
   cd bnpm
   ```
3. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/bnpm-io/bnpm.git
   ```

### Set Up the Development Environment

1. Install [Bun](https://bun.sh):
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```
2. Install [Rust](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. Install system dependencies:
   ```bash
   brew install just buf
   ```
4. Install project dependencies:
   ```bash
   bun install
   ```
5. Build all packages:
   ```bash
   bun run build
   ```

## Development Workflow

### Branch Naming

Use descriptive branch names with a conventional prefix:

- `feat/` — new features (e.g., `feat/add-workspace-support`)
- `fix/` — bug fixes (e.g., `fix/resolve-memory-leak`)
- `chore/` — maintenance (e.g., `chore/update-dependencies`)
- `docs/` — documentation (e.g., `docs/add-api-guide`)
- `refactor/` — code refactoring (e.g., `refactor/extract-parser`)

### Conventional Commits

All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/)
specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `perf`, `ci`, `style`.

Examples:
- `feat(cli): add install command with progress bar`
- `fix(core): handle missing manifest edge case`
- `docs(api): document registry endpoints`

### Pull Request Process

1. Create a feature branch from `main`.
2. Make your changes following the code style guidelines.
3. Write or update tests as needed.
4. Run the full test suite locally.
5. Push your branch and open a pull request.
6. Ensure all CI checks pass.
7. Request review from maintainers.

## Code Style

### TypeScript / JavaScript

We use [Biome](https://biomejs.dev/) for formatting and linting. Configuration
is in `biome.json`.

```bash
# Format code
bun run lint --fix
```

Rules:
- Strict TypeScript mode (`strict: true`)
- No unused variables or parameters
- Path aliases defined in `tsconfig.json`
- Prefer named exports over default exports
- Use `type` imports for type-only imports

### Rust

We use `rustfmt` with the project settings in `rustfmt.toml` and `clippy` with
`clippy::pedantic`.

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --workspace
```

Rules:
- Edition 2024
- No `unsafe` code unless absolutely necessary and reviewed
- Use workspace inheritance for shared dependencies
- Document all public APIs with doc comments

### General Guidelines

- Keep changes focused — one feature or fix per PR.
- Avoid adding unnecessary dependencies.
- Write clear, self-documenting code rather than adding comments.
- Use meaningful variable and function names.
- Keep functions small and focused on a single responsibility.

## Testing

### TypeScript Tests

All TypeScript packages use [Vitest](https://vitest.dev/).

```bash
# Run all TS tests
bun run test

# Run tests for a specific package
bun run test --filter @better-npm/cli

# Run tests in watch mode
bun run test --watch

# Run with coverage
bun run test --coverage
```

### Rust Tests

All Rust crates use the built-in `cargo test` framework.

```bash
# Run all Rust tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p bnpm-core

# Run tests with output
cargo test -- --nocapture
```

### Test Structure

- Place tests in a `__tests__/` directory next to the source for TS.
- Place unit tests in the same file using `#[cfg(test)]` for Rust.
- Use descriptive test names that explain the expected behavior.
- Follow the Arrange-Act-Assert pattern.

## Pull Request Requirements

### PR Title Format

PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

### Labels

Every PR must have at least one of these labels:
- `semver:patch` — bug fixes and minor changes
- `semver:minor` — new features (backward compatible)
- `semver:major` — breaking changes

Additional labels: `needs-review`, `work-in-progress`, `documentation`,
`performance`, `security`.

### Review Process

1. At least one maintainer review is required.
2. All CI checks must pass (lint, test, build, typecheck).
3. Changes may be requested by reviewers.
4. Once approved, a maintainer will merge the PR.

### CI Checks

All PRs run the following checks:
- `lint` — Biome for TS, clippy for Rust
- `test` — Vitest for TS, cargo test for Rust
- `typecheck` — TypeScript type checking
- `build` — Compile all packages and crates

## Issue Reporting

### Bug Reports

Open a [bug report](https://github.com/bnpm-io/bnpm/issues/new?template=bug_report.yml)
with:
- A clear, descriptive title
- Steps to reproduce the behavior
- Expected vs. actual behavior
- Environment details (OS, Bnpm version, Bun version)
- Minimal reproduction case, if possible

### Feature Requests

Open a [feature request](https://github.com/bnpm-io/bnpm/issues/new?template=feature_request.yml)
with:
- A clear, descriptive title
- Use case and motivation
- Proposed solution or API design
- Alternatives you've considered

### Security Issues

Do **not** open a public issue for security vulnerabilities. Please report
security issues privately to **security@bnpm.dev**. See our
[Security Policy](SECURITY.md) for details.

## Project Structure

```
bnpm/
├── apps/           # Application services
│   ├── cli/        # Bnpm CLI binary
│   ├── api/        # Public API server (Hono)
│   ├── web/        # ⟦Bnpmboard⟧ web app (Next.js)
│   └── docs/       # Documentation site
├── crates/         # Rust crates (internal workspace)
│   ├── core/       # Core library
│   ├── parser/     # Manifest parser
│   ├── registry/   # Package registry client
│   └── installer/  # Package installer
├── packages/       # TypeScript packages (published to npm)
│   ├── cli/        # CLI library
│   ├── config/     # Configuration utilities
│   └── types/      # Shared TypeScript types
├── proto/          # Protobuf definitions
├── tools/          # Build and development tools
│   └── generators/ # Code generators (Plop)
├── turbo.json      # Turborepo configuration
└── package.json    # Workspace root
```

## Recognition

All contributors will be recognized in our
[CONTRIBUTORS.md](CONTRIBUTORS.md) file. We value every contribution, whether
it's code, documentation, bug reports, or community support.

---

Thank you for helping make Bnpm better!
