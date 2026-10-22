import { readFileSync } from "node:fs"

export interface BnpmConfig {
  api: {
    url: string
    port: number
    host: string
    corsOrigin: string
  }
  registry: {
    url: string
    port: number
    storagePath: string
  }
  auth: {
    secret: string
    url: string
    issuer: string
    tokenExpiry: string
  }
  telemetry: {
    enabled: boolean
    sentryDsn?: string
    sentryEnvironment?: string
    otlpEndpoint?: string
    serviceName: string
    samplerRatio: number
  }
  storage: {
    s3Endpoint: string
    s3Region: string
    s3Bucket: string
    s3AccessKey: string
    s3SecretKey: string
  }
  cache: {
    redisUrl: string
    defaultTtl: number
  }
  workers: {
    concurrency: number
    queueUrl: string
    timeoutMs: number
  }
  search: {
    indexPath: string
    meiliHttpAddr: string
    meiliMasterKey: string
  }
  logging: {
    level: string
    format: string
    destination: string
  }
  benchmarks: {
    outputResults?: boolean
    outputPath?: string
  }
}

const DEFAULT_CONFIG: BnpmConfig = {
  api: {
    url: "http://localhost:3000",
    port: 3000,
    host: "0.0.0.0",
    corsOrigin: "*",
  },
  registry: {
    url: "http://localhost:4000",
    port: 4000,
    storagePath: "./data/registry",
  },
  auth: {
    secret: "development-secret",
    url: "http://localhost:3000/auth",
    issuer: "bnpm",
    tokenExpiry: "7d",
  },
  telemetry: {
    enabled: false,
    serviceName: "bnpm",
    samplerRatio: 0.1,
  },
  storage: {
    s3Endpoint: "http://localhost:9000",
    s3Region: "us-east-1",
    s3Bucket: "bnpm",
    s3AccessKey: "minioadmin",
    s3SecretKey: "minioadmin",
  },
  cache: {
    redisUrl: "redis://localhost:6379",
    defaultTtl: 3600,
  },
  workers: {
    concurrency: 5,
    queueUrl: "http://localhost:5000",
    timeoutMs: 30000,
  },
  search: {
    indexPath: "./data/search",
    meiliHttpAddr: "http://localhost:7700",
    meiliMasterKey: "master-key",
  },
  logging: {
    level: "info",
    format: "text",
    destination: "stdout",
  },
  benchmarks: {
    outputResults: false,
    outputPath: "benchmarks-results.json",
  },
}

function envBool(key: string, defaultValue: boolean): boolean {
  const val = process.env[key]
  if (val === undefined) return defaultValue
  return val === "true" || val === "1"
}

function envInt(key: string, defaultValue: number): number {
  const val = process.env[key]
  if (val === undefined) return defaultValue
  const parsed = Number.parseInt(val, 10)
  return Number.isNaN(parsed) ? defaultValue : parsed
}

function envStr(key: string, defaultValue: string): string {
  return process.env[key] ?? defaultValue
}

function envConfig(): Partial<BnpmConfig> {
  return {
    api: {
      url: envStr("API_URL", DEFAULT_CONFIG.api.url),
      port: envInt("API_PORT", DEFAULT_CONFIG.api.port),
      host: envStr("API_HOST", DEFAULT_CONFIG.api.host),
      corsOrigin: envStr("API_CORS_ORIGIN", DEFAULT_CONFIG.api.corsOrigin),
    },
    registry: {
      url: envStr("REGISTRY_URL", DEFAULT_CONFIG.registry.url),
      port: envInt("REGISTRY_PORT", DEFAULT_CONFIG.registry.port),
      storagePath: envStr("REGISTRY_STORAGE_PATH", DEFAULT_CONFIG.registry.storagePath),
    },
    auth: {
      secret: envStr("AUTH_SECRET", DEFAULT_CONFIG.auth.secret),
      url: envStr("AUTH_URL", DEFAULT_CONFIG.auth.url),
      issuer: envStr("AUTH_ISSUER", DEFAULT_CONFIG.auth.issuer),
      tokenExpiry: DEFAULT_CONFIG.auth.tokenExpiry,
    },
    telemetry: {
      enabled: envBool("TELEMETRY_ENABLED", DEFAULT_CONFIG.telemetry.enabled),
      sentryDsn: envStr("SENTRY_DSN", ""),
      sentryEnvironment: envStr("SENTRY_ENVIRONMENT", ""),
      otlpEndpoint: envStr("OTEL_EXPORTER_OTLP_ENDPOINT", ""),
      serviceName: envStr("OTEL_SERVICE_NAME", DEFAULT_CONFIG.telemetry.serviceName),
      samplerRatio: envInt("OTEL_SAMPLER_RATIO", DEFAULT_CONFIG.telemetry.samplerRatio),
    },
    storage: {
      s3Endpoint: envStr("S3_ENDPOINT", DEFAULT_CONFIG.storage.s3Endpoint),
      s3Region: envStr("S3_REGION", DEFAULT_CONFIG.storage.s3Region),
      s3Bucket: envStr("S3_BUCKET", DEFAULT_CONFIG.storage.s3Bucket),
      s3AccessKey: envStr("S3_ACCESS_KEY", DEFAULT_CONFIG.storage.s3AccessKey),
      s3SecretKey: envStr("S3_SECRET_KEY", DEFAULT_CONFIG.storage.s3SecretKey),
    },
    cache: {
      redisUrl: envStr("REDIS_URL", DEFAULT_CONFIG.cache.redisUrl),
      defaultTtl: DEFAULT_CONFIG.cache.defaultTtl,
    },
    workers: {
      concurrency: envInt("WORKER_CONCURRENCY", DEFAULT_CONFIG.workers.concurrency),
      queueUrl: envStr("WORKER_QUEUE_URL", DEFAULT_CONFIG.workers.queueUrl),
      timeoutMs: envInt("WORKER_TIMEOUT_MS", DEFAULT_CONFIG.workers.timeoutMs),
    },
    search: {
      indexPath: envStr("SEARCH_INDEX_PATH", DEFAULT_CONFIG.search.indexPath),
      meiliHttpAddr: envStr("MEILI_HTTP_ADDR", DEFAULT_CONFIG.search.meiliHttpAddr),
      meiliMasterKey: envStr("MEILI_MASTER_KEY", DEFAULT_CONFIG.search.meiliMasterKey),
    },
    logging: {
      level: envStr("LOG_LEVEL", DEFAULT_CONFIG.logging.level),
      format: DEFAULT_CONFIG.logging.format,
      destination: DEFAULT_CONFIG.logging.destination,
    },
  }
}

let cachedConfig: BnpmConfig | undefined

export function getEnvVar(name: string, defaultValue?: string): string {
  return process.env[name] ?? defaultValue ?? ""
}

export function isDevelopment(): boolean {
  const nodeEnv = process.env.NODE_ENV
  return !nodeEnv || nodeEnv === "development" || nodeEnv === "dev"
}

export function isProduction(): boolean {
  return process.env.NODE_ENV === "production"
}

function deepMergeConfig(base: BnpmConfig, override: Partial<BnpmConfig>): BnpmConfig {
  const result: Record<string, unknown> = { ...base }
  for (const key of Object.keys(override) as Array<keyof BnpmConfig>) {
    const val = override[key]
    if (val && typeof val === "object" && !Array.isArray(val)) {
      const existing = result[key]
      result[key] = {
        ...(existing as Record<string, unknown>),
        ...(val as Record<string, unknown>),
      }
    } else if (val !== undefined) {
      result[key] = val
    }
  }
  return result as unknown as BnpmConfig
}

export function loadConfig(options?: { file?: string; env?: string }): BnpmConfig {
  let config: BnpmConfig = { ...DEFAULT_CONFIG }

  const envOverrides = envConfig()
  config = deepMergeConfig(config, envOverrides)

  if (options?.file) {
    try {
      const content = readFileSync(options.file, "utf-8")
      const fileConfig = JSON.parse(content) as Partial<BnpmConfig>
      config = deepMergeConfig(config, fileConfig)
    } catch {
      // silently ignore file load errors
    }
  }

  cachedConfig = config
  return config
}

export function getConfig(): BnpmConfig {
  if (!cachedConfig) {
    return loadConfig()
  }
  return cachedConfig
}
