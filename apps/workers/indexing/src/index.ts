import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { createSearchClient } from "@dash/search"
import { createStorageClient } from "@dash/storage"

const logger = createLogger("workers-indexing")
const config = loadConfig()

export interface IndexPayload {
  packageName: string
  packageVersion: string
  action: "index" | "reindex" | "remove"
}

export async function handleIndexing(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const indexPayload = payload as unknown as IndexPayload
  logger.info("Processing indexing job", {
    packageName: indexPayload.packageName,
    packageVersion: indexPayload.packageVersion,
    action: indexPayload.action,
  })

  try {
    const result = await processIndexing(indexPayload)
    return { status: "success", worker: "@dash/workers-indexing", result }
  } catch (error) {
    logger.error("Failed to process indexing job", {
      packageName: indexPayload.packageName,
      action: indexPayload.action,
      error: String(error),
    })
    return { status: "failure", worker: "@dash/workers-indexing", error: String(error) }
  }
}

async function processIndexing(payload: IndexPayload): Promise<{
  indexed: boolean
  documentId?: string
  fields: number
}> {
  const { packageName, packageVersion, action } = payload
  const searchClient = await createSearchClient(config.search)

  if (action === "remove") {
    const documentId = `${packageName}@${packageVersion}`
    await searchClient.remove(documentId)
    logger.info("Document removed from search index", { documentId })
    return { indexed: false, documentId, fields: 0 }
  }

  const storageClient = await createStorageClient()
  const metadata = await storageClient.getMetadata(packageName, packageVersion)

  if (!metadata) {
    throw new Error(`Package metadata not found for ${packageName}@${packageVersion}`)
  }

  const document = {
    id: `${packageName}@${packageVersion}`,
    name: packageName,
    version: packageVersion,
    description: metadata.description ?? "",
    keywords: metadata.keywords ?? [],
    author: metadata.author ?? "",
    license: metadata.license ?? "",
    homepage: metadata.homepage ?? "",
    repository: metadata.repository ?? "",
    readme: metadata.readme ?? "",
    dependencies: Object.keys(metadata.dependencies ?? {}),
    devDependencies: Object.keys(metadata.devDependencies ?? {}),
    updatedAt: new Date().toISOString(),
  }

  if (action === "reindex") {
    await searchClient.remove(document.id)
  }

  const result = await searchClient.index(document.id, document)
  const fieldCount = Object.keys(document).length

  logger.info("Package indexed for search", {
    packageName,
    packageVersion,
    action,
    fields: fieldCount,
    documentId: result.id,
  })

  return { indexed: true, documentId: result.id, fields: fieldCount }
}
