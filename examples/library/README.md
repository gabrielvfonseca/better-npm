# Library Example

A reusable library package built with [tsup](https://tsup.egoist.dev/) that wraps `@better-npm/sdk` utilities for distribution via npm.

## What This Demonstrates

- Building a library that depends on `@better-npm/sdk`
- Dual-format bundling (ESM + CJS) with TypeScript declaration generation
- Structuring a publishable npm package with `main`, `types`, and `exports` fields

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## Setup & Build

```bash
bun install

# Build for production
bun run build

# Watch mode for development
bun run dev
```

The built output goes into `dist/` with:
- `dist/index.js` — ES module
- `dist/index.cjs` — CommonJS
- `dist/index.d.ts` — TypeScript declarations

## File Structure

```
library/
├── package.json        # Package metadata & build scripts
├── src/
│   └── index.ts        # Public API exports
└── tsconfig.json       # TypeScript configuration
```

## Publishing

After building, publish to npm:

```bash
npm publish
```

The `"files": ["dist"]` field in `package.json` ensures only the compiled output (not source) is published.
