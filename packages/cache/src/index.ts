export interface CacheBackend {
  get<T>(key: string): Promise<T | undefined>
  set<T>(key: string, value: T, ttl?: number): Promise<void>
  delete(key: string): Promise<void>
  has(key: string): Promise<boolean>
  clear(): Promise<void>
}

interface CacheEntry<T> {
  value: T
  expiresAt: number | null
}

export class MemoryCache implements CacheBackend {
  private store = new Map<string, CacheEntry<unknown>>()
  private defaultTtl: number | null
  private cleanupInterval: ReturnType<typeof setInterval> | undefined

  constructor(ttl: number | null = null) {
    this.defaultTtl = ttl
    this.cleanupInterval = setInterval(() => this.evictExpired(), 60_000)
  }

  private evictExpired(): void {
    const now = Date.now()
    for (const [key, entry] of this.store) {
      if (entry.expiresAt !== null && entry.expiresAt <= now) {
        this.store.delete(key)
      }
    }
  }

  async get<T>(key: string): Promise<T | undefined> {
    const entry = this.store.get(key)
    if (!entry) return undefined

    if (entry.expiresAt !== null && entry.expiresAt <= Date.now()) {
      this.store.delete(key)
      return undefined
    }

    return entry.value as T
  }

  async set<T>(key: string, value: T, ttl?: number): Promise<void> {
    const expiresAt =
      ttl !== undefined
        ? Date.now() + ttl * 1000
        : this.defaultTtl !== null
          ? Date.now() + this.defaultTtl * 1000
          : null

    this.store.set(key, { value, expiresAt })
  }

  async delete(key: string): Promise<void> {
    this.store.delete(key)
  }

  async has(key: string): Promise<boolean> {
    const entry = this.store.get(key)
    if (!entry) return false

    if (entry.expiresAt !== null && entry.expiresAt <= Date.now()) {
      this.store.delete(key)
      return false
    }

    return true
  }

  async clear(): Promise<void> {
    this.store.clear()
  }

  [Symbol.dispose](): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval)
    }
  }
}

export class RedisCache implements CacheBackend {
  private client: {
    get: (key: string) => Promise<string | null>
    set: (key: string, value: string, ttl?: number) => Promise<unknown>
    del: (key: string) => Promise<unknown>
    exists: (key: string) => Promise<number>
  }

  constructor(client: {
    get: (key: string) => Promise<string | null>
    set: (key: string, value: string, ttl?: number) => Promise<unknown>
    del: (key: string) => Promise<unknown>
    exists: (key: string) => Promise<number>
  }) {
    this.client = client
  }

  async get<T>(key: string): Promise<T | undefined> {
    const value = await this.client.get(key)
    if (value === null) return undefined
    return JSON.parse(value) as T
  }

  async set<T>(key: string, value: T, ttl?: number): Promise<void> {
    const serialized = JSON.stringify(value)
    if (ttl !== undefined) {
      await this.client.set(key, serialized, ttl)
    } else {
      await this.client.set(key, serialized)
    }
  }

  async delete(key: string): Promise<void> {
    await this.client.del(key)
  }

  async has(key: string): Promise<boolean> {
    const result = await this.client.exists(key)
    return result === 1
  }

  async clear(): Promise<void> {
    throw new Error("Redis clear not implemented")
  }
}

export interface CacheConfig {
  backend: "memory" | "redis"
  ttl?: number
  redisClient?: {
    get: (key: string) => Promise<string | null>
    set: (key: string, value: string, ttl?: number) => Promise<unknown>
    del: (key: string) => Promise<unknown>
    exists: (key: string) => Promise<number>
  }
}

export function createCache(config: CacheConfig): CacheBackend {
  switch (config.backend) {
    case "memory":
      return new MemoryCache(config.ttl ?? null)
    case "redis":
      if (!config.redisClient) throw new Error("Redis client required for redis backend")
      return new RedisCache(config.redisClient)
    default:
      throw new Error(`Unknown cache backend: ${config.backend}`)
  }
}

const memoCache = new MemoryCache(null)

export async function memoize<T>(fn: () => Promise<T>, key: string, ttl?: number): Promise<T> {
  const cached = await memoCache.get<T>(key)
  if (cached !== undefined) return cached

  const result = await fn()
  await memoCache.set(key, result, ttl)
  return result
}
