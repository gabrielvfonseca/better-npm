import { z } from "zod"

export const packageNameSchema = z
  .string()
  .min(1)
  .max(214)
  .regex(/^(@[a-z0-9-~][a-z0-9-._~]*\/)?[a-z0-9-~][a-z0-9-._~]*$/, "Invalid package name")

export const semVerSchema = z
  .string()
  .regex(/^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+(?:\+[a-zA-Z0-9.]+)?)?$/, "Invalid semver")

export const packageManifestSchema = z.object({
  name: packageNameSchema,
  version: semVerSchema,
  description: z.string().optional(),
  main: z.string().optional(),
  types: z.string().optional(),
  exports: z.record(z.unknown()).optional(),
  files: z.array(z.string()).optional(),
  scripts: z.record(z.string()).optional(),
  dependencies: z.record(z.string()).optional(),
  devDependencies: z.record(z.string()).optional(),
  peerDependencies: z.record(z.string()).optional(),
  optionalDependencies: z.record(z.string()).optional(),
  bundledDependencies: z.array(z.string()).optional(),
  engines: z.record(z.string()).optional(),
  os: z.array(z.string()).optional(),
  cpu: z.array(z.string()).optional(),
  private: z.boolean().optional(),
  publishConfig: z.record(z.unknown()).optional(),
})

export const publishPackageSchema = z.object({
  name: packageNameSchema,
  version: semVerSchema,
  tag: z.string().default("latest"),
  readme: z.string().optional(),
  access: z.enum(["public", "restricted"]).default("public"),
  dist: z.object({
    tarball: z.string().url(),
    shasum: z.string(),
    integrity: z.string().optional(),
    fileCount: z.number().int().positive().optional(),
    unpackedSize: z.number().int().positive().optional(),
  }),
  dependencies: z.record(z.string()).optional(),
  devDependencies: z.record(z.string()).optional(),
  peerDependencies: z.record(z.string()).optional(),
  optionalDependencies: z.record(z.string()).optional(),
  engines: z.record(z.string()).optional(),
})

export const searchQuerySchema = z.object({
  q: z.string().min(1).max(256),
  scope: z.string().optional(),
  filters: z.record(z.unknown()).optional(),
  sort: z.enum(["relevance", "downloads", "updated", "published"]).optional(),
})

export const paginationParamsSchema = z.object({
  page: z.coerce.number().int().positive().default(1),
  pageSize: z.coerce.number().int().min(1).max(100).default(20),
})

export const createUserSchema = z.object({
  username: z.string().min(2).max(32).regex(/^[a-z0-9_-]+$/),
  email: z.string().email(),
  password: z.string().min(8).max(128),
})

export const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
})

export const updateUserSchema = z.object({
  username: z.string().min(2).max(32).regex(/^[a-z0-9_-]+$/).optional(),
  email: z.string().email().optional(),
  avatarUrl: z.string().url().optional(),
})

export const apiErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.unknown().optional(),
})

export const paginatedResponseSchema = <T extends z.ZodTypeAny>(itemSchema: T) =>
  z.object({
    data: z.array(itemSchema),
    pagination: z.object({
      page: z.number(),
      pageSize: z.number(),
      total: z.number(),
      totalPages: z.number(),
      hasNext: z.boolean(),
      hasPrevious: z.boolean(),
    }),
  })

export const workerPayloadSchema = z.object({
  id: z.string(),
  type: z.string(),
  data: z.record(z.unknown()),
  timestamp: z.string(),
  retryCount: z.number().int().min(0),
})

export const workerResultSchema = z.discriminatedUnion("status", [
  z.object({ status: z.literal("success"), data: z.unknown().optional() }),
  z.object({ status: z.literal("failure"), error: z.string().optional() }),
  z.object({ status: z.literal("retry"), data: z.unknown().optional(), error: z.string().optional() }),
])

export type PublishPackageInput = z.infer<typeof publishPackageSchema>
export type SearchQueryInput = z.infer<typeof searchQuerySchema>
export type PaginationParamsInput = z.infer<typeof paginationParamsSchema>
export type CreateUserInput = z.infer<typeof createUserSchema>
export type LoginInput = z.infer<typeof loginSchema>
export type UpdateUserInput = z.infer<typeof updateUserSchema>
export type ApiErrorInput = z.infer<typeof apiErrorSchema>
export type WorkerPayloadInput = z.infer<typeof workerPayloadSchema>
export type WorkerResultInput = z.infer<typeof workerResultSchema>
export type PackageManifestInput = z.infer<typeof packageManifestSchema>
