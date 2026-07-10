# Node.js Example

A command-line / server script that demonstrates using `@dash/sdk` in a plain [Node.js](https://nodejs.org/) environment with [Express](https://expressjs.com/).

## What This Demonstrates

- Using `@dash/sdk` in a Node.js runtime (not a browser)
- Building a lightweight HTTP server with Express
- Watch-mode development with `tsx`

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## Setup & Run

```bash
bun install

# Development (watch mode)
bun run dev

# Production build & start
bun run build
bun run start
```

## File Structure

```
node/
├── package.json        # Dependencies & scripts
├── src/
│   └── index.ts        # Express server with health & root endpoints
└── tsconfig.json       # TypeScript configuration
```

## API Endpoints

| Route     | Description          |
|-----------|----------------------|
| `GET /`   | Returns a greeting   |
| `GET /health` | Health-check response |
