export type {
  PackageId,
  PackageName,
  SemVer,
  PackageVersion,
  PackageManifest,
  PackageLock,
  LockEntry,
  RegistryPackage,
  PackageVersionInfo,
  DistInfo,
  User,
  UserRole,
  ApiError,
  Result,
  AsyncResult,
  PaginatedResponse,
  PaginationParams,
  WorkerPayload,
  WorkerResult,
  EventType,
} from "@dash/types"

export interface ApiHandler<TInput, TOutput> {
  handle(input: TInput): Promise<TOutput>
}

export interface ApiMiddleware {
  before?: (req: Request) => Promise<Request>
  after?: (res: Response) => Promise<Response>
}

export interface ApiRoute {
  path: string
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
  handler: ApiHandler<unknown, unknown>
  middleware?: ApiMiddleware[]
}

export interface ApiRouter {
  routes: ApiRoute[]
  use(middleware: ApiMiddleware): void
  add(route: ApiRoute): void
}

export interface ApiContext {
  request: Request
  params: Record<string, string>
  query: Record<string, string>
  user?: {
    id: string
    email: string
    roles: string[]
  }
}

export interface ApiResponse<T = unknown> {
  status: number
  body: T
  headers?: Record<string, string>
}

export function createApiResponse<T>(status: number, body: T, headers?: Record<string, string>): ApiResponse<T> {
  return { status, body, headers }
}

export function ok<T>(body: T): ApiResponse<T> {
  return createApiResponse(200, body)
}

export function created<T>(body: T): ApiResponse<T> {
  return createApiResponse(201, body)
}

export function noContent(): ApiResponse<undefined> {
  return createApiResponse(204, undefined)
}

export function badRequest(message: string): ApiResponse<{ code: string; message: string }> {
  return createApiResponse(400, { code: "BAD_REQUEST", message })
}

export function notFound(message: string = "Resource not found"): ApiResponse<{ code: string; message: string }> {
  return createApiResponse(404, { code: "NOT_FOUND", message })
}

export function unauthorized(message: string = "Unauthorized"): ApiResponse<{ code: string; message: string }> {
  return createApiResponse(401, { code: "UNAUTHORIZED", message })
}

export function forbidden(message: string = "Forbidden"): ApiResponse<{ code: string; message: string }> {
  return createApiResponse(403, { code: "FORBIDDEN", message })
}

export function serverError(message: string = "Internal server error"): ApiResponse<{ code: string; message: string }> {
  return createApiResponse(500, { code: "INTERNAL_ERROR", message })
}
