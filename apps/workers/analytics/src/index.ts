import { createLogger } from "@dash/logger"
import { loadConfig } from "@dash/config"
import { createTelemetryClient } from "@dash/telemetry"

const logger = createLogger("workers-analytics")
const config = loadConfig()

export interface AnalyticsEvent {
  name: string
  properties?: Record<string, unknown>
  timestamp?: string
  userId?: string
  sessionId?: string
  source?: string
}

export interface AnalyticsPayload {
  events: AnalyticsEvent[]
  batch?: boolean
}

export async function handleAnalytics(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const analyticsPayload = payload as unknown as AnalyticsPayload
  logger.info("Processing analytics job", {
    eventCount: analyticsPayload.events?.length,
    batch: analyticsPayload.batch,
  })

  try {
    const result = await processAnalytics(analyticsPayload)
    return { status: "success", worker: "@dash/workers-analytics", result }
  } catch (error) {
    logger.error("Failed to process analytics job", { error: String(error) })
    return { status: "failure", worker: "@dash/workers-analytics", error: String(error) }
  }
}

async function processAnalytics(payload: AnalyticsPayload): Promise<{
  processed: number
  failed: number
  aggregated?: Record<string, number>
}> {
  const { events, batch } = payload

  if (!events || events.length === 0) {
    return { processed: 0, failed: 0 }
  }

  const telemetry = await createTelemetryClient(config.telemetry)
  let processed = 0
  let failed = 0
  const aggregated: Record<string, number> = {}

  for (const event of events) {
    try {
      const enrichedEvent = {
        ...event,
        timestamp: event.timestamp ?? new Date().toISOString(),
        processedAt: new Date().toISOString(),
        processor: "@dash/workers-analytics",
      }

      if (batch) {
        aggregated[event.name] = (aggregated[event.name] ?? 0) + 1
      }

      await telemetry.ingestEvent(enrichedEvent)
      processed++
    } catch (error) {
      logger.error("Failed to process analytics event", {
        eventName: event.name,
        error: String(error),
      })
      failed++
    }
  }

  if (batch && Object.keys(aggregated).length > 0) {
    await telemetry.ingestMetrics(
      Object.entries(aggregated).map(([eventName, count]) => ({
        name: `analytics.event.${eventName}`,
        value: count,
        type: "counter",
        timestamp: new Date().toISOString(),
        tags: { batch: "true", processor: "@dash/workers-analytics" },
      })),
    )

    logger.info("Analytics batch aggregation completed", {
      events: processed,
      uniqueEvents: Object.keys(aggregated).length,
      aggregated: aggregated,
    })
  }

  return { processed, failed, aggregated: batch ? aggregated : undefined }
}
