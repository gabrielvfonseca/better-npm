export type PackageId = string & { readonly __brand: unique symbol }
export type PackageName = string & { readonly __brand: unique symbol }
export type SemVer = string & { readonly __brand: unique symbol }
export type PackageVersion = string & { readonly __brand: unique symbol }

export interface PackageManifest {
  name: string
  version: string
  description?: string
  main?: string
  types?: string
  exports?: Record<string, unknown>
  files?: string[]
  scripts?: Record<string, string>
  dependencies?: Record<string, string>
  devDependencies?: Record<string, string>
  peerDependencies?: Record<string, string>
  optionalDependencies?: Record<string, string>
  bundledDependencies?: string[]
  engines?: Record<string, string>
  os?: string[]
  cpu?: string[]
  private?: boolean
  publishConfig?: Record<string, unknown>
}

export interface PackageLock {
  name: string
  version: string
  lockfileVersion: number
  packages: Record<string, LockEntry>
}

export interface LockEntry {
  version: string
  resolved?: string
  integrity?: string
  dependencies?: Record<string, string>
  optionalDependencies?: Record<string, string>
  dev?: boolean
}

export interface RegistryPackage {
  id: string
  name: string
  versions: Record<string, PackageVersionInfo>
  description?: string
  distTags: Record<string, string>
  homepage?: string
  repository?: Record<string, string>
  license?: string
  author?: string | Record<string, string>
  readme?: string
  updatedAt: string
  createdAt: string
}

export interface PackageVersionInfo {
  name: string
  version: string
  description?: string
  main?: string
  types?: string
  dist: DistInfo
  dependencies?: Record<string, string>
  devDependencies?: Record<string, string>
  peerDependencies?: Record<string, string>
  optionalDependencies?: Record<string, string>
  engines?: Record<string, string>
  hasInstallScript?: boolean
  deprecated?: string
}

export interface DistInfo {
  tarball: string
  shasum: string
  integrity?: string
  fileCount?: number
  unpackedSize?: number
  signatures?: Array<{ keyid: string; sig: string }>
}

export interface User {
  id: string
  username: string
  email: string
  avatarUrl?: string
  createdAt: string
  roles: UserRole[]
}

export enum UserRole {
  Admin = "admin",
  Moderator = "moderator",
  Publisher = "publisher",
  User = "user",
}

export interface ApiError {
  code: string
  message: string
  details?: unknown
}

export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E }

export type AsyncResult<T, E = Error> = Promise<Result<T, E>>

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    pageSize: number
    total: number
    totalPages: number
    hasNext: boolean
    hasPrevious: boolean
  }
}

export interface PaginationParams {
  page?: number
  pageSize?: number
}

export interface WorkerPayload {
  id: string
  type: string
  data: Record<string, unknown>
  timestamp: string
  retryCount: number
}

export type WorkerResult =
  | { status: "success"; data?: unknown }
  | { status: "failure"; error?: string }
  | { status: "retry"; data?: unknown; error?: string }

export type EventType =
  | "package:publish"
  | "package:unpublish"
  | "package:deprecate"
  | "user:login"
  | "user:register"
  | "search:index"
  | "search:reindex"
