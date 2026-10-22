import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
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
      body: typeof data === "string" ? data : new Uint8Array(data),
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

export interface PackageDist {
  tarball: string
  shasum: string
  integrity?: string
  fileCount?: number
  unpackedSize?: number
}

export interface PublishPackagePayload {
  name: string
  version: string
  tag?: string
  readme?: string
  access?: "public" | "restricted"
  dist: PackageDist
  dependencies?: Record<string, string>
  devDependencies?: Record<string, string>
  peerDependencies?: Record<string, string>
  optionalDependencies?: Record<string, string>
  engines?: Record<string, string>
}

export interface PackageVersionRecord extends Omit<PublishPackagePayload, "tag" | "access"> {
  tag: string
  access: "public" | "restricted"
  publishedAt: string
  deprecated?: string
}

export interface PackageRecord {
  name: string
  owners: string[]
  distTags: Record<string, string>
  versions: Record<string, PackageVersionRecord>
  deprecated?: string
}

export class PackageStoreError extends Error {
  constructor(
    readonly status: 409,
    message: string,
  ) {
    super(message)
  }
}

export interface PackageStorage {
  getPackage(name: string): Promise<PackageRecord | undefined>
  getPackageVersion(name: string, version: string): Promise<PackageVersionRecord | undefined>
  publishPackage(
    name: string,
    payload: PublishPackagePayload,
    publisher: string,
  ): Promise<PackageRecord>
  unpublishPackageVersion(name: string, version: string): Promise<boolean>
  deprecatePackage(name: string, reason: string): Promise<PackageRecord | undefined>
  deprecatePackageVersion(
    name: string,
    version: string,
    reason: string,
  ): Promise<PackageVersionRecord | undefined>
  searchPackages(query: string): Promise<PackageRecord[]>
}

class JsonPackageStorage implements PackageStorage {
  private readonly packages: Record<string, PackageRecord>

  constructor(private readonly file: string) {
    this.packages = existsSync(file)
      ? (JSON.parse(readFileSync(file, "utf-8")) as Record<string, PackageRecord>)
      : {}
  }

  private save(): void {
    mkdirSync(dirname(this.file), { recursive: true })
    const tmp = `${this.file}.tmp`
    writeFileSync(tmp, JSON.stringify(this.packages, null, 2))
    renameSync(tmp, this.file)
  }

  async getPackage(name: string): Promise<PackageRecord | undefined> {
    return this.packages[name]
  }

  async getPackageVersion(
    name: string,
    version: string,
  ): Promise<PackageVersionRecord | undefined> {
    return this.packages[name]?.versions[version]
  }

  async publishPackage(
    name: string,
    payload: PublishPackagePayload,
    publisher: string,
  ): Promise<PackageRecord> {
    const pkg: PackageRecord = this.packages[name] ?? {
      name,
      owners: [publisher],
      distTags: {},
      versions: {},
    }
    if (pkg.versions[payload.version]) {
      throw new PackageStoreError(
        409,
        `Version ${payload.version} of '${name}' is already published`,
      )
    }
    const tag = payload.tag ?? "latest"
    pkg.versions[payload.version] = {
      ...payload,
      tag,
      access: payload.access ?? "public",
      publishedAt: new Date().toISOString(),
    }
    pkg.distTags[tag] = payload.version
    this.packages[name] = pkg
    this.save()
    return pkg
  }

  async unpublishPackageVersion(name: string, version: string): Promise<boolean> {
    const pkg = this.packages[name]
    if (!pkg?.versions[version]) return false
    delete pkg.versions[version]
    for (const [tag, v] of Object.entries(pkg.distTags)) {
      if (v === version) delete pkg.distTags[tag]
    }
    this.save()
    return true
  }

  async deprecatePackage(name: string, reason: string): Promise<PackageRecord | undefined> {
    const pkg = this.packages[name]
    if (!pkg) return undefined
    pkg.deprecated = reason
    this.save()
    return pkg
  }

  async deprecatePackageVersion(
    name: string,
    version: string,
    reason: string,
  ): Promise<PackageVersionRecord | undefined> {
    const record = this.packages[name]?.versions[version]
    if (!record) return undefined
    record.deprecated = reason
    this.save()
    return record
  }

  async searchPackages(query: string): Promise<PackageRecord[]> {
    const needle = query.toLowerCase()
    return Object.values(this.packages).filter((pkg) => pkg.name.toLowerCase().includes(needle))
  }
}

export const PackageStorageFactory = {
  create(file = process.env.BNPM_PACKAGE_STORE ?? "./data/registry/packages.json"): PackageStorage {
    return new JsonPackageStorage(file)
  },
}
