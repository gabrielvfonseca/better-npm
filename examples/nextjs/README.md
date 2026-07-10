# Next.js Example

A [Next.js](https://nextjs.org/) 15 application using the App Router that demonstrates using `@dash/sdk` for server-side data fetching.

## What This Demonstrates

- Using `@dash/sdk` with Next.js 15 App Router
- Server-side data fetching in an `async` Server Component
- `next.config.ts` rewrites to proxy API calls to the Dash registry
- Incremental Static Regeneration via `next: { revalidate: 60 }`

## Prerequisites

- [Bun](https://bun.sh/) or [pnpm](https://pnpm.io/) / [npm](https://www.npmjs.com/)
- [Node.js](https://nodejs.org/) 22+

## Setup & Run

```bash
bun install
bun run dev
```

Open http://localhost:3000 in your browser.

## File Structure

```
nextjs/
├── app/
│   └── page.tsx        # Home page (async Server Component)
├── next.config.ts      # Next.js config with rewrites
├── package.json        # Dependencies & scripts
└── tsconfig.json       # TypeScript configuration
```

## Rewrites / Proxy

`next.config.ts` defines a rewrite rule so that `/api/:path*` requests are proxied to the Dash registry at `http://localhost:3000`. This keeps the client/server API path consistent without exposing the registry URL directly.

```ts
rewrites: async () => [
  {
    source: '/api/:path*',
    destination: 'http://localhost:3000/api/:path*',
  },
],
```
