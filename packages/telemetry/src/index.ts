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

export function recordMetric(
  _name: string,
  _value: number,
  _attributes?: Record<string, string>,
): void {
  if (isShutdown) return
  // attributes intentionally unused in this implementation
}

export function incrementCounter(
  _name: string,
  _value = 1,
  _attributes?: Record<string, string>,
): void {
  if (isShutdown) return
  // attributes intentionally unused in this implementation
}

export function recordHistogram(
  _name: string,
  _value: number,
  _attributes?: Record<string, string>,
): void {
  if (isShutdown) return
  // attributes intentionally unused in this implementation
}

export function setAttribute(_key: string, _value: string): void {
  if (isShutdown) return
  // key intentionally unused in this implementation
  // value intentionally unused in this implementation
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

// Additional telemetry client for health checks
export interface TelemetryClient {
  ingestEvents(events: { name: string; timestamp: number }[]): Promise<{ accepted: boolean }[]>
  ingestMetrics(metrics: { name: string; value: number }[]): Promise<{ accepted: boolean }[]>
  isConnected(): boolean
  exportPrometheus(): Promise<string>
  checkDatabase(): Promise<{ connected: boolean }>
}

export class TelemetryClientImpl implements TelemetryClient {
  async ingestEvents(
    _events: { name: string; timestamp: number }[],
  ): Promise<{ accepted: boolean }[]> {
    // No-op implementation: accept all events
    return _events.map(() => ({ accepted: true }))
  }

  async ingestMetrics(
    _metrics: { name: string; value: number }[],
  ): Promise<{ accepted: boolean }[]> {
    // No-op implementation: accept all metrics
    return _metrics.map(() => ({ accepted: true }))
  }

  isConnected(): boolean {
    // Assume connected for now
    return true
  }

  async exportPrometheus(): Promise<string> {
    // Return an empty Prometheus metrics string
    return ""
  }

  async checkDatabase(): Promise<{ connected: boolean }> {
    // In a real implementation, this would check actual database connectivity
    // For now, return healthy as a placeholder
    return { connected: true }
  }
}

export function createTelemetryClient(): TelemetryClient {
  return new TelemetryClientImpl()
}
