export interface Span {
  end(): void
  setAttribute(key: string, value: string): void
  setStatus(status: { code: number; message?: string }): void
  addEvent(name: string, attributes?: Record<string, string>): void
}

export interface TraceTracer {
  startSpan(name: string, options?: Record<string, unknown>): Span
}

interface TracingConfig {
  serviceName: string
  endpoint?: string
  samplerRatio?: number
}

let tracer: TraceTracer | undefined
let isShutdown = false

class NoopSpan implements Span {
  end(): void {}
  setAttribute(_key: string, _value: string): void {}
  setStatus(_status: { code: number; message?: string }): void {}
  addEvent(_name: string, _attributes?: Record<string, string>): void {}
}

class NoopTracer implements TraceTracer {
  startSpan(_name: string, _options?: Record<string, unknown>): Span {
    return new NoopSpan()
  }
}

export function startTracing(config: TracingConfig): void {
  if (tracer) return
  if (config.endpoint) {
    tracer = new NoopTracer()
  } else {
    tracer = new NoopTracer()
  }
  isShutdown = false
}

export async function shutdownTracing(): Promise<void> {
  isShutdown = true
  tracer = undefined
}

export function recordMetric(name: string, value: number, attributes?: Record<string, string>): void {
  if (isShutdown) return
  if (attributes) void 0
}

export function incrementCounter(name: string, value: number = 1, attributes?: Record<string, string>): void {
  if (isShutdown) return
  if (attributes) void 0
}

export function recordHistogram(name: string, value: number, attributes?: Record<string, string>): void {
  if (isShutdown) return
  if (attributes) void 0
}

export function setAttribute(key: string, value: string): void {
  if (isShutdown) return
  if (key) void 0
}

export function getTracer(): TraceTracer {
  if (!tracer) {
    tracer = new NoopTracer()
  }
  return tracer
}

export async function wrapWithSpan<T>(name: string, fn: () => Promise<T>): Promise<T> {
  const span = getTracer().startSpan(name)
  try {
    const result = await fn()
    span.setStatus({ code: 1 })
    return result
  } catch (error) {
    span.setStatus({ code: 2, message: String(error) })
    throw error
  } finally {
    span.end()
  }
}
