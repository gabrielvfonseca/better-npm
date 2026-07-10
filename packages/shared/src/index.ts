export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export function randomId(length: number = 16): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789"
  let result = ""
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)]
  }
  return result
}

export function omit<T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  ...keys: K[]
): Omit<T, K> {
  const result = { ...obj }
  for (const key of keys) {
    delete result[key]
  }
  return result as Omit<T, K>
}

export function pick<T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  ...keys: K[]
): Pick<T, K> {
  const result = {} as Pick<T, K>
  for (const key of keys) {
    if (key in obj) {
      result[key] = obj[key]
    }
  }
  return result
}

export function deepMerge<T extends Record<string, unknown>>(
  target: T,
  source: Partial<T>,
): T {
  const result = { ...target } as Record<string, unknown>
  for (const key of Object.keys(source as Record<string, unknown>)) {
    const val = (source as Record<string, unknown>)[key]
    if (
      val &&
      typeof val === "object" &&
      !Array.isArray(val) &&
      result[key] &&
      typeof result[key] === "object" &&
      !Array.isArray(result[key])
    ) {
      result[key] = deepMerge(
        result[key] as Record<string, unknown>,
        val as Record<string, unknown>,
      )
    } else {
      result[key] = val
    }
  }
  return result as T
}

export interface RetryOptions {
  retries?: number
  delay?: number
  backoff?: boolean
}

export async function retry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {},
): Promise<T> {
  const { retries: maxRetries = 3, delay: baseDelay = 1000, backoff = true } = options
  let lastError: unknown
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error
      if (attempt < maxRetries) {
        const wait = backoff ? baseDelay * 2 ** attempt : baseDelay
        await sleep(wait)
      }
    }
  }
  throw lastError
}

export function timeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error(`Timed out after ${ms}ms`)), ms)
  })
  return Promise.race([promise, timeoutPromise])
}

export function chunk<T>(array: T[], size: number): T[][] {
  if (size <= 0) throw new Error("Size must be positive")
  const result: T[][] = []
  for (let i = 0; i < array.length; i += size) {
    result.push(array.slice(i, i + size))
  }
  return result
}

export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | undefined
  return (...args: Parameters<T>) => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => {
      fn(...args)
      timer = undefined
    }, delay)
  }
}

export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let lastCall = 0
  let timer: ReturnType<typeof setTimeout> | undefined

  return (...args: Parameters<T>) => {
    const now = Date.now()
    const remaining = delay - (now - lastCall)

    if (remaining <= 0) {
      lastCall = now
      fn(...args)
    } else if (!timer) {
      timer = setTimeout(() => {
        lastCall = Date.now()
        fn(...args)
        timer = undefined
      }, remaining)
    }
  }
}

type EventHandler = (...args: unknown[]) => void

export class EventEmitter {
  private events = new Map<string, Set<EventHandler>>()

  on(event: string, handler: EventHandler): void {
    if (!this.events.has(event)) {
      this.events.set(event, new Set())
    }
    this.events.get(event)!.add(handler)
  }

  off(event: string, handler: EventHandler): void {
    this.events.get(event)?.delete(handler)
  }

  emit(event: string, ...args: unknown[]): void {
    for (const handler of this.events.get(event) ?? []) {
      handler(...args)
    }
  }

  once(event: string, handler: EventHandler): void {
    const wrapper: EventHandler = (...args) => {
      this.off(event, wrapper)
      handler(...args)
    }
    this.on(event, wrapper)
  }
}
