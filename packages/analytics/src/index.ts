export interface AnalyticsEvent {
  name: string
  properties?: Record<string, unknown>
  timestamp: string
  sessionId: string
  userId?: string
}

export interface AnalyticsConfig {
  endpoint?: string
  apiKey?: string
  flushInterval?: number
  maxQueueSize?: number
  enabled?: boolean
}

export class AnalyticsClient {
  private config: Required<AnalyticsConfig>
  private queue: AnalyticsEvent[] = []
  private flushTimer: ReturnType<typeof setInterval> | undefined
  private sessionId: string

  constructor(config: AnalyticsConfig = {}) {
    this.config = {
      endpoint: config.endpoint ?? "https://analytics.dash.dev/events",
      apiKey: config.apiKey ?? "",
      flushInterval: config.flushInterval ?? 30000,
      maxQueueSize: config.maxQueueSize ?? 100,
      enabled: config.enabled ?? true,
    }
    this.sessionId = this.generateSessionId()

    if (this.config.enabled) {
      this.flushTimer = setInterval(() => {
        void this.flush()
      }, this.config.flushInterval)
    }
  }

  private generateSessionId(): string {
    return `sess_${crypto.randomUUID().slice(0, 8)}`
  }

  track(event: AnalyticsEvent): void {
    if (!this.config.enabled) return

    this.queue.push({
      ...event,
      timestamp: event.timestamp ?? new Date().toISOString(),
      sessionId: event.sessionId ?? this.sessionId,
    })

    if (this.queue.length >= this.config.maxQueueSize) {
      void this.flush()
    }
  }

  trackPageView(path: string, properties?: Record<string, unknown>): void {
    this.track({
      name: "page_view",
      properties: { path, ...properties },
      timestamp: new Date().toISOString(),
      sessionId: this.sessionId,
    })
  }

  identify(userId: string, traits?: Record<string, unknown>): void {
    this.track({
      name: "identify",
      properties: { userId, traits },
      timestamp: new Date().toISOString(),
      sessionId: this.sessionId,
      userId,
    })
  }

  async flush(): Promise<void> {
    if (this.queue.length === 0) return

    const batch = this.queue.splice(0)
    try {
      const response = await fetch(this.config.endpoint, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
        },
        body: JSON.stringify({ events: batch }),
      })

      if (!response.ok) {
        this.queue.unshift(...batch)
      }
    } catch {
      this.queue.unshift(...batch)
    }
  }

  [Symbol.dispose](): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer)
    }
  }
}

export function createAnalyticsClient(config?: AnalyticsConfig): AnalyticsClient {
  return new AnalyticsClient(config)
}
