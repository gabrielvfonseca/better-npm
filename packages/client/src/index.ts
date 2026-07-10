export interface DashClientConfig {
  baseUrl: string
  token?: string
  timeout?: number
}

export interface RequestOptions {
  headers?: Record<string, string>
  timeout?: number
  retries?: number
}

export interface ApiErrorResponse {
  code: string
  message: string
  details?: unknown
}

export class DashClientError extends Error {
  code: string
  status: number
  details?: unknown

  constructor(status: number, code: string, message: string, details?: unknown) {
    super(message)
    this.name = "DashClientError"
    this.status = status
    this.code = code
    this.details = details
  }
}

export class DashClient {
  private config: DashClientConfig
  private token: string | undefined

  constructor(config: DashClientConfig) {
    this.config = config
    this.token = config.token
  }

  setToken(token: string | undefined): void {
    this.token = token
  }

  getToken(): string | undefined {
    return this.token
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    options: RequestOptions = {},
  ): Promise<T> {
    const url = `${this.config.baseUrl}${path}`
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...options.headers,
    }

    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`
    }

    let lastError: Error | undefined

    for (let attempt = 0; attempt <= (options.retries ?? 0); attempt++) {
      try {
        const controller = new AbortController()
        const timeoutMs = options.timeout ?? this.config.timeout ?? 30000
        const timeoutId = setTimeout(() => controller.abort(), timeoutMs)

        const response = await fetch(url, {
          method,
          headers,
          body: body ? JSON.stringify(body) : undefined,
          signal: controller.signal,
        })

        clearTimeout(timeoutId)

        if (!response.ok) {
          const errorBody = await response.json().catch(() => ({})) as Partial<ApiErrorResponse>
          throw new DashClientError(
            response.status,
            errorBody.code ?? "UNKNOWN_ERROR",
            errorBody.message ?? response.statusText,
            errorBody.details,
          )
        }

        if (response.status === 204) {
          return undefined as T
        }

        return (await response.json()) as T
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error))
        if (error instanceof DashClientError) throw error
        if (attempt < (options.retries ?? 0)) {
          const delay = 1000 * 2 ** attempt
          await new Promise((resolve) => setTimeout(resolve, delay))
        }
      }
    }

    throw lastError ?? new Error("Request failed")
  }

  async get<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>("GET", path, undefined, options)
  }

  async post<T>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return this.request<T>("POST", path, body, options)
  }

  async put<T>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return this.request<T>("PUT", path, body, options)
  }

  async delete<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>("DELETE", path, undefined, options)
  }

  async patch<T>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return this.request<T>("PATCH", path, body, options)
  }
}

export function createClient(config: DashClientConfig): DashClient {
  return new DashClient(config)
}
