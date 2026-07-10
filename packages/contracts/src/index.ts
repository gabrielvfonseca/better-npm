export interface ApiContract<TInput, TOutput> {
  path: string
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
  input: TInput
  output: TOutput
}

export function createContractRoute<TInput, TOutput>(
  contract: ApiContract<TInput, TOutput>,
): ApiContract<TInput, TOutput> {
  return contract
}

// Package endpoints
export const getPackage = createContractRoute({
  path: "/v1/packages/:name",
  method: "GET",
  input: {} as { name: string },
  output: {} as { name: string; version: string; description?: string; readme?: string },
})

export const listPackages = createContractRoute({
  path: "/v1/packages",
  method: "GET",
  input: {} as { page?: number; pageSize?: number; scope?: string },
  output: {} as {
    data: Array<{ name: string; version: string; description?: string }>
    pagination: { page: number; pageSize: number; total: number; totalPages: number; hasNext: boolean; hasPrevious: boolean }
  },
})

export const publishPackage = createContractRoute({
  path: "/v1/packages",
  method: "POST",
  input: {} as {
    name: string
    version: string
    tag?: string
    readme?: string
    access?: "public" | "restricted"
    dist: { tarball: string; shasum: string }
    dependencies?: Record<string, string>
    devDependencies?: Record<string, string>
    peerDependencies?: Record<string, string>
    optionalDependencies?: Record<string, string>
    engines?: Record<string, string>
  },
  output: {} as { success: true; name: string; version: string },
})

export const deprecatePackage = createContractRoute({
  path: "/v1/packages/:name/:version",
  method: "DELETE",
  input: {} as { name: string; version: string; message?: string },
  output: {} as { success: true },
})

export const unpublishPackage = createContractRoute({
  path: "/v1/packages/:name/:version",
  method: "DELETE",
  input: {} as { name: string; version: string; force?: boolean },
  output: {} as { success: true },
})

// Search endpoints
export const searchPackages = createContractRoute({
  path: "/v1/search",
  method: "GET",
  input: {} as { q: string; page?: number; pageSize?: number; sort?: string },
  output: {} as {
    hits: Array<{ id: string; name: string; description?: string; score: number }>
    total: number
    page: number
    pageSize: number
  },
})

export const getSuggestions = createContractRoute({
  path: "/v1/search/suggest",
  method: "GET",
  input: {} as { q: string },
  output: {} as { suggestions: Array<{ text: string; score: number }> },
})

// Auth endpoints
export const login = createContractRoute({
  path: "/v1/auth/login",
  method: "POST",
  input: {} as { email: string; password: string },
  output: {} as { token: string; user: { id: string; email: string; username: string } },
})

export const register = createContractRoute({
  path: "/v1/auth/register",
  method: "POST",
  input: {} as { username: string; email: string; password: string },
  output: {} as { token: string; user: { id: string; email: string; username: string } },
})

export const getProfile = createContractRoute({
  path: "/v1/auth/profile",
  method: "GET",
  input: {} as Record<string, never>,
  output: {} as { id: string; username: string; email: string; avatarUrl?: string; createdAt: string; roles: string[] },
})

export const updateProfile = createContractRoute({
  path: "/v1/auth/profile",
  method: "PATCH",
  input: {} as { username?: string; email?: string; avatarUrl?: string },
  output: {} as { id: string; username: string; email: string; avatarUrl?: string },
})

// Stats endpoints
export const getDownloads = createContractRoute({
  path: "/v1/stats/downloads",
  method: "GET",
  input: {} as { package?: string; period?: "day" | "week" | "month" | "year" },
  output: {} as { downloads: Array<{ date: string; count: number }>; total: number },
})

export const getStats = createContractRoute({
  path: "/v1/stats",
  method: "GET",
  input: {} as Record<string, never>,
  output: {} as {
    totalPackages: number
    totalVersions: number
    totalDownloads: number
    totalUsers: number
  },
})
