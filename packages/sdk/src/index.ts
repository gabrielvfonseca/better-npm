import {
  type AuthSession,
  type AuthToken,
  createToken,
  hashPassword,
  parseAuthHeader,
  verifyPassword,
  verifyToken,
} from "@better-npm/auth"
import { type BnpmClient, type BnpmClientConfig, createClient } from "@better-npm/client"
import {
  type BnpmConfig,
  getConfig,
  getEnvVar,
  isDevelopment,
  isProduction,
  loadConfig,
} from "@better-npm/config"
import { type Logger, createLogger } from "@better-npm/logger"
import {
  createUserSchema,
  loginSchema,
  packageManifestSchema,
  packageNameSchema,
  paginationParamsSchema,
  publishPackageSchema,
  searchQuerySchema,
  semVerSchema,
  updateUserSchema,
} from "@better-npm/schemas"
import type {
  ApiError,
  EventType,
  PackageLock,
  PackageManifest,
  PaginatedResponse,
  PaginationParams,
  RegistryPackage,
  Result,
  User,
  WorkerPayload,
  WorkerResult,
} from "@better-npm/types"

export type {
  BnpmClientConfig,
  PackageManifest,
  PackageLock,
  RegistryPackage,
  User,
  PaginatedResponse,
  PaginationParams,
  WorkerPayload,
  WorkerResult,
  EventType,
  Result,
  ApiError,
  Logger,
  BnpmConfig,
  AuthToken,
  AuthSession,
}

export interface BnpmOptions {
  baseUrl?: string
  token?: string
  config?: Partial<BnpmConfig>
}

export class Bnpm {
  client: BnpmClient
  logger: Logger
  config: BnpmConfig

  schemas = {
    packageName: packageNameSchema,
    semVer: semVerSchema,
    packageManifest: packageManifestSchema,
    publishPackage: publishPackageSchema,
    searchQuery: searchQuerySchema,
    paginationParams: paginationParamsSchema,
    createUser: createUserSchema,
    login: loginSchema,
    updateUser: updateUserSchema,
  }

  auth = {
    verifyToken,
    createToken,
    hashPassword,
    verifyPassword,
    parseAuthHeader,
  }

  env = {
    get: getEnvVar,
    isDev: isDevelopment,
    isProd: isProduction,
  }

  constructor(options?: BnpmOptions) {
    const baseUrl = options?.baseUrl ?? getEnvVar("API_URL", "http://localhost:3000")
    const token = options?.token ?? getEnvVar("AUTH_TOKEN", "")

    this.config = getConfig()
    this.logger = createLogger("bnpm:sdk")

    this.client = createClient({
      baseUrl,
      token: token || undefined,
    })
  }
}

export function createBnpm(options?: BnpmOptions): Bnpm {
  return new Bnpm(options)
}

export { loadConfig, getConfig, getEnvVar, isDevelopment, isProduction }
