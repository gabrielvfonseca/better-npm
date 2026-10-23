import { mkdtempSync, rmSync } from "node:fs"
import http from "node:http"
import { tmpdir } from "node:os"
import { join } from "node:path"
import type { BnpmConfig } from "@better-npm/config"
import { Logger } from "@better-npm/logger"
import type { PackageManifest } from "@better-npm/types"

export function createTestLogger(): Logger {
  return new Logger("test")
}

export function createTestConfig(overrides?: Partial<BnpmConfig>): BnpmConfig {
  const config: BnpmConfig = {
    api: { url: "http://localhost:3000", port: 3000, host: "0.0.0.0", corsOrigin: "*" },
    registry: { url: "http://localhost:4000", port: 4000, storagePath: "/tmp/bnpm-test/registry" },
    auth: {
      secret: "test-secret",
      url: "http://localhost:3000/auth",
      issuer: "bnpm-test",
      tokenExpiry: "1h",
    },
    telemetry: { enabled: false, serviceName: "bnpm-test", samplerRatio: 0 },
    storage: { s3Endpoint: "", s3Region: "", s3Bucket: "", s3AccessKey: "", s3SecretKey: "" },
    cache: { redisUrl: "", defaultTtl: 60 },
    workers: { concurrency: 1, queueUrl: "", timeoutMs: 5000 },
    search: { indexPath: "/tmp/bnpm-test/search", meiliHttpAddr: "", meiliMasterKey: "" },
    logging: { level: "silent", format: "json", destination: "stdout" },
    benchmarks: { outputResults: false, outputPath: "benchmarks-results.json" },
  }

  if (overrides) {
    return deepMerge(config, overrides)
  }

  return config
}

function deepMerge<T extends object>(target: T, source: Partial<T>): T {
  const result = { ...target } as Record<string, unknown>
  for (const key of Object.keys(source) as Array<keyof T>) {
    const val = source[key]
    if (val && typeof val === "object" && !Array.isArray(val)) {
      result[key as string] = {
        ...(result[key as string] as Record<string, unknown>),
        ...(val as Record<string, unknown>),
      }
    } else if (val !== undefined) {
      result[key as string] = val
    }
  }
  return result as T
}

export type MockHandler = (req: Request) => Response | Promise<Response>

export async function createMockServer(handlers: Record<string, MockHandler>): Promise<{
  url: string
  close: () => void
}> {
  const routes = new Map<string, MockHandler>()
  for (const [path, handler] of Object.entries(handlers)) {
    routes.set(path, handler)
  }

  const server = http.createServer(async (req, res) => {
    const url = new URL(req.url ?? "http://localhost/")
    const handler = routes.get(url.pathname) ?? routes.get("*")
    if (handler) {
      const response = await handler(req as unknown as Request)
      res.writeHead(response.status, Object.fromEntries(response.headers.entries()))
      const body = await response.arrayBuffer()
      res.end(Buffer.from(body))
    } else {
      res.writeHead(404, { "Content-Type": "text/plain" })
      res.end("Not found")
    }
  })

  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address()
      const port = typeof addr === "object" && addr ? addr.port : 0
      resolve({
        url: `http://127.0.0.1:${port}`,
        close: () => server.close(),
      })
    })
  })
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export function randomId(): string {
  return `test_${crypto.randomUUID().slice(0, 8)}`
}

export function createTestPackage(overrides?: Partial<PackageManifest>): PackageManifest {
  const pkg: PackageManifest = {
    name: `@better-npm/test-${randomId()}`,
    version: "1.0.0",
    description: "A test package",
    main: "./src/index.js",
    types: "./src/index.d.ts",
  }

  return { ...pkg, ...overrides }
}

export async function withTempDir(fn: (dir: string) => Promise<void>): Promise<void> {
  const dir = mkdtempSync(join(tmpdir(), "bnpm-test-"))
  try {
    await fn(dir)
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}
