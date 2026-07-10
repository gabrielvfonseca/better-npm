import { DashClient, createClient, type DashClientConfig } from "@dash/client"
import type { PackageManifest, PackageLock, RegistryPackage, User, PaginatedResponse, PaginationParams, WorkerPayload, WorkerResult, EventType, Result, ApiError } from "@dash/types"
import { createLogger, type Logger } from "@dash/logger"
import { loadConfig, getConfig, getEnvVar, isDevelopment, isProduction, type DashConfig } from "@dash/config"
import { verifyToken, createToken, hashPassword, verifyPassword, parseAuthHeader, type AuthToken, type AuthSession } from "@dash/auth"
import { packageNameSchema, semVerSchema, packageManifestSchema, publishPackageSchema, searchQuerySchema, paginationParamsSchema, createUserSchema, loginSchema, updateUserSchema } from "@dash/schemas"

export type {
  DashClientConfig,
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
  DashConfig,
  AuthToken,
  AuthSession,
}

export interface DashOptions {
  baseUrl?: string
  token?: string
  config?: Partial<DashConfig>
}

export class Dash {
  client: DashClient
  logger: Logger
  config: DashConfig

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

  constructor(options?: DashOptions) {
    const baseUrl = options?.baseUrl ?? getEnvVar("API_URL", "http://localhost:3000")
    const token = options?.token ?? getEnvVar("AUTH_TOKEN", "")

    this.config = getConfig()
    this.logger = createLogger("dash:sdk")

    this.client = createClient({
      baseUrl,
      token: token || undefined,
    })
  }
}

export function createDash(options?: DashOptions): Dash {
  return new Dash(options)
}

export {
  loadConfig,
  getConfig,
  getEnvVar,
  isDevelopment,
  isProduction,
}
