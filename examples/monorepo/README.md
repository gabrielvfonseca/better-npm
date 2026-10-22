# Monorepo Example

A minimal two-package monorepo using [Turborepo](https://turbo.build/repo) that demonstrates sharing a common library (`@better-npm/example-shared`) between apps.

## What This Demonstrates

- Structuring a monorepo with `packages/` (shared lib) and `apps/` (consumer)
- Using npm/pnpm/bun workspaces for local package resolution
- Turborepo pipeline for orchestrating `build` and `dev` across packages

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## How to Run

```bash
bun install

# Run the dev script for all packages
bun run dev

# Build all packages in dependency order
bun run build
```

## File Structure

```
monorepo/
├── apps/
│   └── web/                    # Consumer app
│       ├── package.json        # Depends on @better-npm/example-shared
│       └── src/
│           └── index.ts        # Uses shared utilities
├── packages/
│   └── shared/                 # Shared library
│       ├── package.json        # Depends on @better-npm/sdk
│       └── src/
│           └── index.ts        # Exports sharedUtil, getPackageUrl
├── package.json                # Root workspace & turbo config
├── turbo.json                  # Turborepo pipeline definitions
└── tsconfig.json               # Shared TypeScript config
```

## How It Works

- `packages/shared` exports utilities (`sharedUtil`, `getPackageUrl`) that wrap `@better-npm/sdk`
- `apps/web` imports from `@better-npm/example-shared` (resolved via workspace protocol)
- Turborepo runs tasks in dependency order: `shared` builds before `web`
- The root `package.json` uses `"workspaces": ["packages/*", "apps/*"]` to register both directories
