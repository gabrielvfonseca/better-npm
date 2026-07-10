import type { Logger } from "@dash/logger"
import type { DashConfig } from "@dash/config"
import type { PackageManifest } from "@dash/types"
import { mkdtempSync, writeFileSync, rmSync } from "node:fs"
import { join } from "node:path"
import { tmpdir } from "node:os"

export function createTestLogger(): Logger {
  const levels = ["debug", "info", "warn", "error", "fatal"]
  const logger = { module: "test" } as Logger

  for (const level of levels) {
    (logger as Record<string, unknown>)[level] = () => {}
  }

  logger.child = () => createTestLogger()

  return logger
}

export function createTestConfig(overrides?: Partial<DashConfig>): DashConfig {
  const config: DashConfig = {
    api: { url: "http://localhost:3000", port: 3000, host: "0.0.0.0", corsOrigin: "*" },
    registry: { url: "http://localhost:4000", port: 4000, storagePath: "/tmp/dash-test/registry" },
    auth: { secret: "test-secret", url: "http://localhost:3000/auth", issuer: "dash-test", tokenExpiry: "1h" },
    telemetry: { enabled: false, serviceName: "dash-test", samplerRatio: 0 },
    storage: { s3Endpoint: "", s3Region: "", s3Bucket: "", s3AccessKey: "", s3SecretKey: "" },
    cache: { redisUrl: "", defaultTtl: 60 },
    workers: { concurrency: 1, queueUrl: "", timeoutMs: 5000 },
    search: { indexPath: "/tmp/dash-test/search", meiliHttpAddr: "", meiliMasterKey: "" },
    logging: { level: "silent", format: "json", destination: "stdout" },
  }

  if (overrides) {
    return deepMerge(config, overrides)
  }

  return config
}

function deepMerge<T extends Record<string, unknown>>(target: T, source: Partial<T>): T {
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

export interface MockHandler {
  (req: Request): Response | Promise<Response>
}

export function createMockServer(
  handlers: Record<string, MockHandler>,
): { url: string; close: () => void } {
  const routes = new Map<string, MockHandler>()
  for (const [path, handler] of Object.entries(handlers)) {
    routes.set(path, handler)
  }

  const server = Bun.serve({
    port: 0,
    async fetch(req) {
      const url = new URL(req.url)
      const handler = routes.get(url.pathname) ?? routes.get("*")
      if (handler) {
        return handler(req)
      }
      return new Response("Not found", { status: 404 })
    },
  })

  return {
    url: `http://localhost:${server.port}`,
    close: () => server.stop(),
  }
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export function randomId(): string {
  return `test_${crypto.randomUUID().slice(0, 8)}`
}

export function createTestPackage(overrides?: Partial<PackageManifest>): PackageManifest {
  const pkg: PackageManifest = {
    name: `@dash/test-${randomId()}`,
    version: "1.0.0",
    description: "A test package",
    main: "./src/index.js",
    types: "./src/index.d.ts",
  }

  return { ...pkg, ...overrides }
}

export async function withTempDir(
  fn: (dir: string) => Promise<void>,
): Promise<void> {
  const dir = mkdtempSync(join(tmpdir(), "dash-test-"))
  try {
    await fn(dir)
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}
