import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { createStorageClient } from "@dash/storage"

const logger = createLogger("workers-metadata")
const _config = loadConfig()

export interface MetadataPayload {
  packageName: string
  packageVersion: string
  fields?: string[]
}

export async function handleMetadata(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const metadataPayload = payload as unknown as MetadataPayload
  logger.info("Processing metadata job", {
    packageName: metadataPayload.packageName,
    packageVersion: metadataPayload.packageVersion,
  })

  try {
    const result = await processMetadata(metadataPayload)
    return { status: "success", worker: "@dash/workers-metadata", result }
  } catch (error) {
    logger.error("Failed to process metadata job", {
      packageName: metadataPayload.packageName,
      error: String(error),
    })
    return { status: "failure", worker: "@dash/workers-metadata", error: String(error) }
  }
}

async function processMetadata(payload: MetadataPayload): Promise<{
  packageName: string
  packageVersion: string
  metadata: Record<string, unknown>
  enriched: boolean
  fieldsExtracted: number
}> {
  const { packageName, packageVersion, fields } = payload
  const storageClient = await createStorageClient()

  const rawMetadata = await storageClient.getRawMetadata(packageName, packageVersion)
  if (!rawMetadata) {
    throw new Error(`Raw metadata not found for ${packageName}@${packageVersion}`)
  }

  const enriched: Record<string, unknown> = {
    ...rawMetadata,
    extractedAt: new Date().toISOString(),
    extractor: "@dash/workers-metadata",
  }

  if (rawMetadata.repository) {
    enriched.repositoryInfo = await extractRepositoryInfo(rawMetadata.repository as string)
  }

  if (rawMetadata.license) {
    enriched.licenseInfo = await extractLicenseInfo(rawMetadata.license as string)
  }

  if (Array.isArray(rawMetadata.keywords)) {
    enriched.normalizedKeywords = (rawMetadata.keywords as string[]).map((k: string) =>
      k.toLowerCase().trim(),
    )
  }

  if (rawMetadata.dependencies) {
    enriched.dependencyGraph = buildDependencyGraph(
      rawMetadata.dependencies as Record<string, string>,
    )
    enriched.totalDependencies = Object.keys(
      rawMetadata.dependencies as Record<string, string>,
    ).length
  }

  if (typeof rawMetadata.readme === "string") {
    enriched.readmeStats = {
      length: (rawMetadata.readme as string).length,
      lines: (rawMetadata.readme as string).split("\n").length,
      hasExamples: /```[\s\S]*?```/.test(rawMetadata.readme as string),
      hasBadges: /!\[.*?\]\(.*?\)/.test(rawMetadata.readme as string),
    }
  }

  const requestedFields = fields ?? Object.keys(enriched)
  const filteredMetadata: Record<string, unknown> = {}
  for (const field of requestedFields) {
    if (field in enriched) {
      filteredMetadata[field] = enriched[field]
    }
  }

  await storageClient.storeMetadata(packageName, packageVersion, filteredMetadata)

  logger.info("Metadata extracted and enriched", {
    packageName,
    packageVersion,
    fieldsExtracted: Object.keys(filteredMetadata).length,
    enriched: true,
  })

  return {
    packageName,
    packageVersion,
    metadata: filteredMetadata,
    enriched: true,
    fieldsExtracted: Object.keys(filteredMetadata).length,
  }
}

async function extractRepositoryInfo(repository: string): Promise<Record<string, string> | null> {
  const match = repository.match(/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?$/)
  if (match) {
    const [, owner, repo] = match
    return {
      provider: "github",
      owner,
      repo,
      url: `https://github.com/${owner}/${repo}`,
      apiUrl: `https://api.github.com/repos/${owner}/${repo}`,
    }
  }
  return null
}

async function extractLicenseInfo(license: string): Promise<Record<string, unknown> | null> {
  const spdxLicenses: Record<string, string> = {
    MIT: "MIT License",
    "Apache-2.0": "Apache License 2.0",
    GPL: "GNU General Public License",
    "GPL-3.0": "GNU General Public License v3.0",
    "GPL-2.0": "GNU General Public License v2.0",
    ISC: "ISC License",
    BSD: "BSD License",
    "BSD-2-Clause": "BSD 2-Clause License",
    "BSD-3-Clause": "BSD 3-Clause License",
    "MPL-2.0": "Mozilla Public License 2.0",
    Unlicense: "The Unlicense",
    "CC0-1.0": "Creative Commons Zero v1.0 Universal",
  }

  return {
    id: license,
    name: spdxLicenses[license] ?? `Unknown (${license})`,
    isSpdx: license in spdxLicenses,
  }
}

function buildDependencyGraph(
  dependencies: Record<string, string>,
): Array<{ name: string; version: string }> {
  return Object.entries(dependencies).map(([name, version]) => ({
    name,
    version: version.replace(/^[\^~]/, ""),
  }))
}
