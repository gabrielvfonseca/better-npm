# React Example

A minimal [React 19](https://react.dev/) + [Vite](https://vite.dev/) application that demonstrates using `@dash/sdk` to fetch and display package data from a Dash registry.

## What This Demonstrates

- Using `@dash/sdk` in a Vite + React 19 browser app
- Fetching package data from a Dash registry via `createDashClient`
- Rendering reactive UI state with `useState` / `useEffect`

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## Setup & Run

```bash
# Install dependencies
bun install

# Start the dev server
bun run dev
```

Open http://localhost:5173 in your browser.

## File Structure

```
react/
├── index.html          # HTML entry point
├── package.json        # Dependencies & scripts
├── src/
│   ├── main.tsx        # React DOM mount
│   └── App.tsx         # Main component (fetches & displays a package)
└── vite.config.ts      # Vite configuration
```

## How It Works

`App.tsx` calls `createDashClient` from `@dash/sdk` to initialise a client pointed at the Dash registry. On mount it fetches package metadata (e.g. lodash) and renders the name, description, and latest version.

The underlying API call is proxied through the Vite dev server to `http://localhost:3000` (the Dash registry).
