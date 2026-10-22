import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { createStorageClient } from "@better-npm/storage"

const logger = createLogger("workers-cleanup")
const _config = loadConfig()

export interface CleanupPayload {
  olderThan?: string
  types?: string[]
}

interface CleanupStats {
  type: string
  removed: number
  bytesFreed: number
  duration: number
}

export async function handleCleanup(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const cleanupPayload = payload as unknown as CleanupPayload
  logger.info("Processing cleanup job", {
    olderThan: cleanupPayload.olderThan,
    types: cleanupPayload.types,
  })

  try {
    const result = await processCleanup(cleanupPayload)
    return { status: "success", worker: "@better-npm/workers-cleanup", result }
  } catch (error) {
    logger.error("Failed to process cleanup job", { error: String(error) })
    return { status: "failure", worker: "@better-npm/workers-cleanup", error: String(error) }
  }
}

async function processCleanup(payload: CleanupPayload): Promise<{
  cleaned: boolean
  types: string[]
  stats: CleanupStats[]
  totalRemoved: number
  totalBytesFreed: number
  totalDuration: number
}> {
  const { olderThan, types } = payload
  const storageClient = await createStorageClient()

  const cleanupTypes = types ?? ["cache", "temp", "old-versions", "logs", "orphaned-metadata"]
  const cutoffDate = olderThan
    ? new Date(olderThan)
    : new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
  const stats: CleanupStats[] = []

  for (const type of cleanupTypes) {
    const start = performance.now()

    try {
      let removed = 0
      let bytesFreed = 0

      switch (type) {
        case "cache":
          ;({ removed, bytesFreed } = await cleanupCache(storageClient, cutoffDate))
          break
        case "temp":
          ;({ removed, bytesFreed } = await cleanupTemp(storageClient, cutoffDate))
          break
        case "old-versions":
          ;({ removed, bytesFreed } = await cleanupOldVersions(storageClient, cutoffDate))
          break
        case "logs":
          ;({ removed, bytesFreed } = await cleanupLogs(storageClient, cutoffDate))
          break
        case "orphaned-metadata":
          ;({ removed, bytesFreed } = await cleanupOrphanedMetadata(storageClient, cutoffDate))
          break
        default:
          logger.warn("Unknown cleanup type", { type })
          continue
      }

      const duration = performance.now() - start
      stats.push({ type, removed, bytesFreed, duration })

      logger.info("Cleanup completed", {
        type,
        removed,
        bytesFreed,
        duration: `${duration.toFixed(0)}ms`,
      })
    } catch (error) {
      logger.error("Cleanup failed for type", { type, error: String(error) })
      stats.push({ type, removed: 0, bytesFreed: 0, duration: 0 })
    }
  }

  const totalRemoved = stats.reduce((sum, s) => sum + s.removed, 0)
  const totalBytesFreed = stats.reduce((sum, s) => sum + s.bytesFreed, 0)
  const totalDuration = stats.reduce((sum, s) => sum + s.duration, 0)

  return {
    cleaned: true,
    types: cleanupTypes,
    stats,
    totalRemoved,
    totalBytesFreed,
    totalDuration,
  }
}

async function cleanupCache(
  storageClient: ReturnType<typeof createStorageClient>,
  cutoffDate: Date,
): Promise<{ removed: number; bytesFreed: number }> {
  const staleEntries = await storageClient.findStaleCacheEntries(cutoffDate)
  let removed = 0
  let bytesFreed = 0

  for (const entry of staleEntries) {
    try {
      const size = entry.size ?? 0
      await storageClient.deleteCacheEntry(entry.key)
      removed++
      bytesFreed += size
    } catch (error) {
      logger.error("Failed to delete cache entry", { key: entry.key, error: String(error) })
    }
  }

  return { removed, bytesFreed }
}

async function cleanupTemp(
  storageClient: ReturnType<typeof createStorageClient>,
  cutoffDate: Date,
): Promise<{ removed: number; bytesFreed: number }> {
  const tempFiles = await storageClient.listTempFiles(cutoffDate)
  let removed = 0
  let bytesFreed = 0

  for (const file of tempFiles) {
    try {
      const size = file.size ?? 0
      await storageClient.deleteTempFile(file.path)
      removed++
      bytesFreed += size
    } catch (error) {
      logger.error("Failed to delete temp file", { path: file.path, error: String(error) })
    }
  }

  return { removed, bytesFreed }
}

async function cleanupOldVersions(
  storageClient: ReturnType<typeof createStorageClient>,
  cutoffDate: Date,
): Promise<{ removed: number; bytesFreed: number }> {
  const oldVersions = await storageClient.findOldPackageVersions(cutoffDate)
  let removed = 0
  let bytesFreed = 0

  for (const version of oldVersions) {
    try {
      const size = version.size ?? 0
      await storageClient.deletePackageVersion(version.packageName, version.version)
      removed++
      bytesFreed += size
    } catch (error) {
      logger.error("Failed to delete old package version", {
        package: `${version.packageName}@${version.version}`,
        error: String(error),
      })
    }
  }

  return { removed, bytesFreed }
}

async function cleanupLogs(
  storageClient: ReturnType<typeof createStorageClient>,
  cutoffDate: Date,
): Promise<{ removed: number; bytesFreed: number }> {
  const oldLogs = await storageClient.findOldLogs(cutoffDate)
  let removed = 0
  let bytesFreed = 0

  for (const log of oldLogs) {
    try {
      const size = log.size ?? 0
      await storageClient.deleteLog(log.id)
      removed++
      bytesFreed += size
    } catch (error) {
      logger.error("Failed to delete log", { id: log.id, error: String(error) })
    }
  }

  return { removed, bytesFreed }
}

async function cleanupOrphanedMetadata(
  storageClient: ReturnType<typeof createStorageClient>,
  cutoffDate: Date,
): Promise<{ removed: number; bytesFreed: number }> {
  const orphaned = await storageClient.findOrphanedMetadata(cutoffDate)
  let removed = 0
  let bytesFreed = 0

  for (const meta of orphaned) {
    try {
      const size = meta.size ?? 0
      await storageClient.deleteMetadata(meta.packageName, meta.packageVersion)
      removed++
      bytesFreed += size
    } catch (error) {
      logger.error("Failed to delete orphaned metadata", {
        package: `${meta.packageName}@${meta.packageVersion}`,
        error: String(error),
      })
    }
  }

  return { removed, bytesFreed }
}
