import { createHash } from "node:crypto"
import { createWriteStream } from "node:fs"
import { mkdir } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { Readable } from "node:stream"
import { pipeline } from "node:stream/promises"
import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { createStorageClient } from "@better-npm/storage"

const logger = createLogger("workers-downloads")
const _config = loadConfig()

export interface DownloadPayload {
  packageName: string
  packageVersion: string
  url: string
  integrity: string
  destination: string
}

export async function handleDownload(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const downloadPayload = payload as unknown as DownloadPayload
  logger.info("Processing download job", {
    packageName: downloadPayload.packageName,
    packageVersion: downloadPayload.packageVersion,
  })

  try {
    const result = await processDownload(downloadPayload)
    return { status: "success", worker: "@better-npm/workers-downloads", result }
  } catch (error) {
    logger.error("Failed to process download job", {
      packageName: downloadPayload.packageName,
      error: String(error),
    })
    return { status: "failure", worker: "@better-npm/workers-downloads", error: String(error) }
  }
}

async function processDownload(payload: DownloadPayload): Promise<{
  downloaded: boolean
  verified: boolean
  cached: boolean
  path: string
  size: number
}> {
  const { packageName, packageVersion, url, integrity, destination } = payload

  const cacheDir = join(
    destination || join(tmpdir(), "bnpm-cache", "packages"),
    packageName,
    packageVersion,
  )
  await mkdir(cacheDir, { recursive: true })

  const filename = `${packageName}-${packageVersion}.tgz`
  const filePath = join(cacheDir, filename)

  logger.info("Downloading package", { url, destination: filePath })

  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`Failed to download package: HTTP ${response.status} ${response.statusText}`)
  }

  const contentLength = response.headers.get("content-length")
  const expectedSize = contentLength ? Number.parseInt(contentLength, 10) : undefined

  const arrayBuffer = await response.arrayBuffer()
  const buffer = Buffer.from(arrayBuffer)
  const actualSize = buffer.length

  if (expectedSize && actualSize !== expectedSize) {
    throw new Error(`Size mismatch: expected ${expectedSize}, got ${actualSize}`)
  }

  const hash = createHash("sha512")
  hash.update(buffer)
  const computedIntegrity = `sha512-${hash.digest("base64")}`

  if (integrity && computedIntegrity !== integrity) {
    throw new Error(`Integrity check failed: expected ${integrity}, computed ${computedIntegrity}`)
  }

  const writeStream = createWriteStream(filePath)
  await pipeline(Readable.from(buffer), writeStream)

  const storageClient = await createStorageClient()
  await storageClient.storeFile(`${packageName}/${packageVersion}/${filename}`, buffer)

  logger.info("Package downloaded and cached", {
    packageName,
    packageVersion,
    size: actualSize,
    path: filePath,
    verified: true,
  })

  return {
    downloaded: true,
    verified: true,
    cached: true,
    path: filePath,
    size: actualSize,
  }
}
