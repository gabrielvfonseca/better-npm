import type { z } from "zod"

export class ValidationError extends Error {
  issues: z.ZodIssue[]

  constructor(issues: z.ZodIssue[]) {
    super("Validation failed")
    this.name = "ValidationError"
    this.issues = issues
  }
}

export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E }

export function validate<T>(
  schema: z.ZodSchema<T>,
  data: unknown,
): Result<T, ValidationError> {
  const result = schema.safeParse(data)
  if (result.success) {
    return { ok: true, value: result.data }
  }
  return { ok: false, error: new ValidationError(result.error.issues) }
}

export function assert<T>(schema: z.ZodSchema<T>, data: unknown): T {
  const result = schema.safeParse(data)
  if (!result.success) {
    throw new ValidationError(result.error.issues)
  }
  return result.data
}

export function isValid(schema: z.ZodSchema, data: unknown): boolean {
  return schema.safeParse(data).success
}

export {
  packageNameSchema,
  semVerSchema,
  packageManifestSchema,
  publishPackageSchema,
  searchQuerySchema,
  paginationParamsSchema,
  createUserSchema,
  loginSchema,
  updateUserSchema,
  apiErrorSchema,
  paginatedResponseSchema,
  workerPayloadSchema,
  workerResultSchema,
} from "@dash/schemas"

export type {
  PublishPackageInput,
  SearchQueryInput,
  PaginationParamsInput,
  CreateUserInput,
  LoginInput,
  UpdateUserInput,
  ApiErrorInput,
  PackageManifestInput,
} from "@dash/schemas"
