import { createLogger } from "@dash/logger"
import { loadConfig } from "@dash/config"
import { createStorageClient } from "@dash/storage"
import { createHash, verify } from "node:crypto"

const logger = createLogger("workers-signatures")
const config = loadConfig()

export interface SignaturePayload {
  packageName: string
  packageVersion: string
  signature: string
  publicKey: string
}

interface VerificationResult {
  packageName: string
  packageVersion: string
  valid: boolean
  algorithm: string
  signedBy?: string
  signedAt?: string
  details?: string
}

export async function handleSignatures(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const sigPayload = payload as unknown as SignaturePayload
  logger.info("Processing signature verification job", {
    packageName: sigPayload.packageName,
    packageVersion: sigPayload.packageVersion,
  })

  try {
    const result = await processSignature(sigPayload)
    return { status: "success", worker: "@dash/workers-signatures", result }
  } catch (error) {
    logger.error("Failed to process signature verification job", {
      packageName: sigPayload.packageName,
      error: String(error),
    })
    return { status: "failure", worker: "@dash/workers-signatures", error: String(error) }
  }
}

async function processSignature(payload: SignaturePayload): Promise<VerificationResult> {
  const { packageName, packageVersion, signature, publicKey } = payload
  const storageClient = await createStorageClient()

  const packageContent = await storageClient.getPackageArchive(packageName, packageVersion)
  if (!packageContent) {
    throw new Error(`Package archive not found for ${packageName}@${packageVersion}`)
  }

  const contentHash = createHash("sha512").update(packageContent).digest("hex")
  const contentBuffer = Buffer.from(contentHash, "hex")
  const signatureBuffer = Buffer.from(signature, "base64")
  const keyBuffer = Buffer.from(publicKey, "utf-8")

  let isValid = false
  let algorithm = "unknown"

  try {
    isValid = verify(
      null,
      contentBuffer,
      {
        key: keyBuffer,
        padding: 1,
      },
      signatureBuffer,
    )
    algorithm = "rsa-sha512"
  } catch {
    try {
      isValid = verify(
        "sha512",
        contentBuffer,
        keyBuffer,
        signatureBuffer,
      )
      algorithm = "ecdsa-sha512"
    } catch {
      throw new Error("Unable to verify signature: unsupported key type or invalid format")
    }
  }

  logger.info("Signature verification completed", {
    packageName,
    packageVersion,
    valid: isValid,
    algorithm,
  })

  const result: VerificationResult = {
    packageName,
    packageVersion,
    valid: isValid,
    algorithm,
    details: isValid ? "Signature verified successfully" : "Signature verification failed",
  }

  if (isValid) {
    try {
      const keyFingerprint = createHash("sha256").update(keyBuffer).digest("hex").slice(0, 16)
      result.signedBy = keyFingerprint
      result.signedAt = new Date().toISOString()
    } catch {
      // Key fingerprint extraction is best-effort
    }
  }

  await storageClient.storeVerificationResult(packageName, packageVersion, result)

  return result
}
