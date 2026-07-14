import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "node:fs"
import { dirname, join } from "node:path"

export interface StorageBackend {
  read(path: string): Promise<Buffer>
  write(path: string, data: Buffer | string): Promise<void>
  delete(path: string): Promise<void>
  list(prefix: string): Promise<string[]>
  exists(path: string): Promise<boolean>
}

export interface S3StorageConfig {
  endpoint: string
  region: string
  bucket: string
  accessKey: string
  secretKey: string
}

export class LocalStorage implements StorageBackend {
  private baseDir: string

  constructor(baseDir: string) {
    this.baseDir = baseDir
    mkdirSync(baseDir, { recursive: true })
  }

  private resolve(path: string): string {
    const resolved = join(this.baseDir, path)
    if (!resolved.startsWith(this.baseDir)) {
      throw new Error("Path traversal denied")
    }
    mkdirSync(dirname(resolved), { recursive: true })
    return resolved
  }

  async read(path: string): Promise<Buffer> {
    return readFileSync(this.resolve(path))
  }

  async write(path: string, data: Buffer | string): Promise<void> {
    writeFileSync(this.resolve(path), data)
  }

  async delete(path: string): Promise<void> {
    try {
      unlinkSync(this.resolve(path))
    } catch {
      // ignore if file doesn't exist
    }
  }

  async list(prefix: string): Promise<string[]> {
    const dir = this.resolve(prefix)
    try {
      const entries = readdirSync(dir)
      return entries
        .filter((entry) => statSync(join(dir, entry)).isFile())
        .map((entry) => join(prefix, entry))
    } catch {
      return []
    }
  }

  async exists(path: string): Promise<boolean> {
    return existsSync(this.resolve(path))
  }
}

export class S3Storage implements StorageBackend {
  private config: S3StorageConfig

  constructor(config: S3StorageConfig) {
    this.config = config
  }

  private get endpoint(): string {
    return `${this.config.endpoint}/${this.config.bucket}`
  }

  async read(path: string): Promise<Buffer> {
    const response = await fetch(`${this.endpoint}/${path}`, {
      headers: this.authHeaders(),
    })
    if (!response.ok) throw new Error(`S3 read failed: ${response.statusText}`)
    const arrayBuffer = await response.arrayBuffer()
    return Buffer.from(arrayBuffer)
  }

  async write(path: string, data: Buffer | string): Promise<void> {
    const response = await fetch(`${this.endpoint}/${path}`, {
      method: "PUT",
      headers: {
        ...this.authHeaders(),
        "Content-Type": "application/octet-stream",
      },
      body: data instanceof Buffer ? data : data,
    })
    if (!response.ok) throw new Error(`S3 write failed: ${response.statusText}`)
  }

  async delete(path: string): Promise<void> {
    const response = await fetch(`${this.endpoint}/${path}`, {
      method: "DELETE",
      headers: this.authHeaders(),
    })
    if (!response.ok) throw new Error(`S3 delete failed: ${response.statusText}`)
  }

  async list(prefix: string): Promise<string[]> {
    const params = new URLSearchParams({ prefix })
    const response = await fetch(`${this.endpoint}?${params}`, {
      headers: this.authHeaders(),
    })
    if (!response.ok) throw new Error(`S3 list failed: ${response.statusText}`)
    const data = (await response.json()) as { contents?: Array<{ key: string }> }
    return (data.contents ?? []).map((c) => c.key)
  }

  async exists(path: string): Promise<boolean> {
    const response = await fetch(`${this.endpoint}/${path}`, {
      method: "HEAD",
      headers: this.authHeaders(),
    })
    return response.ok
  }

  private authHeaders(): Record<string, string> {
    const encoded = btoa(`${this.config.accessKey}:${this.config.secretKey}`)
    return { Authorization: `Basic ${encoded}` }
  }
}

export type StorageType = "local" | "s3"

export interface StorageConfig {
  type: StorageType
  baseDir?: string
  s3?: S3StorageConfig
}

export function createStorage(config: StorageConfig): StorageBackend {
  switch (config.type) {
    case "local":
      return new LocalStorage(config.baseDir ?? "./data/storage")
    case "s3":
      if (!config.s3) throw new Error("S3 configuration required")
      return new S3Storage(config.s3)
    default:
      throw new Error(`Unknown storage type: ${config.type}`)
  }
}
