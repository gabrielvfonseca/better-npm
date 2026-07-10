# Hono Example

A [Hono](https://hono.dev/) server application that demonstrates using `@dash/config` to serve a lightweight API.

## What This Demonstrates

- Using `@dash/config` in a Hono framework server
- Hono's routing, JSON responses, and dynamic path parameters
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
hono/
├── package.json        # Dependencies & scripts
├── src/
│   └── index.ts        # Hono app with routes
└── tsconfig.json       # TypeScript configuration
```

## API Endpoints

| Route         | Description                         |
|---------------|-------------------------------------|
| `GET /`       | Returns a greeting JSON response    |
| `GET /health` | Health-check with service metadata  |
| `GET /api/:path` | Echoes the path segment with a timestamp |
