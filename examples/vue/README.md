# Vue Example

A [Vue 3](https://vuejs.org/) + [Vite](https://vite.dev/) application that demonstrates using `@dash/sdk` with the Composition API and `<script setup>`.

## What This Demonstrates

- Using `@dash/sdk` in a Vue 3 + Vite app
- Vue Composition API with `<script setup lang="ts">`
- Async data fetching with `onMounted` and reactive state via `ref`
- TypeScript interfaces for package metadata

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
vue/
├── index.html          # HTML entry point
├── package.json        # Dependencies & scripts
├── src/
│   ├── main.ts         # Vue app creation & mount
│   └── App.vue         # Root component (Composition API, fetches a package)
└── vite.config.ts      # Vite configuration
```

## Key Points

- `App.vue` uses `<script setup lang="ts">` for concise Composition API syntax
- `ref<Package | null>(null)` holds the reactive package state
- Data is fetched inside `onMounted` and assigned to `pkg.value`
- The template conditionally renders with `v-if="pkg"`
- The API call is proxied through Vite to the Dash registry at `http://localhost:3000`
