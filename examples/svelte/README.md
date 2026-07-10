# Svelte Example

A [Svelte 5](https://svelte.dev/) + [Vite](https://vite.dev/) application that demonstrates using `@dash/sdk` with Svelte 5 runes (`$state`, `$effect`).

## What This Demonstrates

- Using `@dash/sdk` in a Svelte 5 + Vite browser app
- Svelte 5 runes: `$state` for reactive declarations, `$effect` for side effects
- The new Svelte 5 `mount` API in `main.ts`

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## Setup & Run

```bash
bun install
bun run dev
```

Open http://localhost:5173 in your browser.

## File Structure

```
svelte/
├── index.html          # HTML entry point
├── package.json        # Dependencies & scripts
├── src/
│   ├── main.ts         # App mount (Svelte 5 mount API)
│   └── App.svelte      # Root component (runes-based reactivity)
└── vite.config.ts      # Vite configuration
```

## Key Points

- `$state` replaces `let` for reactive variables
- `$effect` replaces `onMount` / `$:` for reactive side effects
- The Svelte 5 `mount()` function in `main.ts` replaces the legacy `new App()` constructor
- API calls are proxied through Vite to the Dash registry at `http://localhost:3000`
