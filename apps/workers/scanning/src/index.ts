import { createLogger } from "@dash/logger"
import { loadConfig } from "@dash/config"
import { createStorageClient } from "@dash/storage"

const logger = createLogger("workers-scanning")
const config = loadConfig()

export interface ScanPayload {
  packageName: string
  packageVersion: string
  files?: string[]
}

interface Vulnerability {
  type: string
  severity: "critical" | "high" | "medium" | "low"
  file: string
  line?: number
  message: string
  recommendation?: string
}

export async function handleScanning(payload: Record<string, unknown>): Promise<{
  status: string
  worker: string
  result?: unknown
  error?: string
}> {
  const scanPayload = payload as unknown as ScanPayload
  logger.info("Processing scanning job", {
    packageName: scanPayload.packageName,
    packageVersion: scanPayload.packageVersion,
  })

  try {
    const result = await processScanning(scanPayload)
    return { status: "success", worker: "@dash/workers-scanning", result }
  } catch (error) {
    logger.error("Failed to process scanning job", {
      packageName: scanPayload.packageName,
      error: String(error),
    })
    return { status: "failure", worker: "@dash/workers-scanning", error: String(error) }
  }
}

async function processScanning(payload: ScanPayload): Promise<{
  packageName: string
  packageVersion: string
  vulnerabilities: Vulnerability[]
  scannedFiles: number
  summary: { critical: number; high: number; medium: number; low: number; total: number }
}> {
  const { packageName, packageVersion, files } = payload
  const storageClient = await createStorageClient()

  const packageFiles = files ?? (await storageClient.listFiles(packageName, packageVersion))
  const vulnerabilities: Vulnerability[] = []

  for (const file of packageFiles) {
    const content = await storageClient.getFileContent(packageName, packageVersion, file)
    if (!content) continue

    const fileVulnerabilities = await scanFile(file, content)
    vulnerabilities.push(...fileVulnerabilities)
  }

  const summary = {
    critical: vulnerabilities.filter((v) => v.severity === "critical").length,
    high: vulnerabilities.filter((v) => v.severity === "high").length,
    medium: vulnerabilities.filter((v) => v.severity === "medium").length,
    low: vulnerabilities.filter((v) => v.severity === "low").length,
    total: vulnerabilities.length,
  }

  await storageClient.storeScanResults(packageName, packageVersion, {
    vulnerabilities,
    summary,
    scannedAt: new Date().toISOString(),
    scanner: "@dash/workers-scanning",
  })

  logger.info("Package scan completed", {
    packageName,
    packageVersion,
    scannedFiles: packageFiles.length,
    vulnerabilities: summary.total,
  })

  return {
    packageName,
    packageVersion,
    vulnerabilities,
    scannedFiles: packageFiles.length,
    summary,
  }
}

async function scanFile(filename: string, content: string): Promise<Vulnerability[]> {
  const vulnerabilities: Vulnerability[] = []
  const lines = content.split("\n")

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!
    const lineNumber = i + 1

    const evalVuln = checkEvalUsage(line, filename, lineNumber)
    if (evalVuln) vulnerabilities.push(evalVuln)

    const innerHTMLVuln = checkInnerHTML(line, filename, lineNumber)
    if (innerHTMLVuln) vulnerabilities.push(innerHTMLVuln)

    const hardcodedSecret = checkHardcodedSecrets(line, filename, lineNumber)
    if (hardcodedSecret) vulnerabilities.push(hardcodedSecret)

    const cmdInjection = checkCommandInjection(line, filename, lineNumber)
    if (cmdInjection) vulnerabilities.push(cmdInjection)

    const prototypePollution = checkPrototypePollution(line, filename, lineNumber)
    if (prototypePollution) vulnerabilities.push(prototypePollution)
  }

  if (filename.endsWith("package.json")) {
    const packageJsonVulns = await analyzeDependencies(content, filename)
    vulnerabilities.push(...packageJsonVulns)
  }

  return vulnerabilities
}

function checkEvalUsage(line: string, file: string, lineNumber: number): Vulnerability | null {
  if (/\beval\s*\(/.test(line) || /\bFunction\s*\(/.test(line)) {
    return {
      type: "eval-usage",
      severity: "high",
      file,
      line: lineNumber,
      message: "Use of eval() or dynamic Function constructor can lead to code injection",
      recommendation: "Avoid using eval(). Use safer alternatives like JSON.parse() or proper parsers.",
    }
  }
  return null
}

function checkInnerHTML(line: string, file: string, lineNumber: number): Vulnerability | null {
  if (/\.innerHTML\s*=/.test(line)) {
    return {
      type: "innerhtml-xss",
      severity: "high",
      file,
      line: lineNumber,
      message: "Setting innerHTML can lead to XSS vulnerabilities",
      recommendation: "Use textContent or insertAdjacentText() instead, or sanitize input with DOMPurify.",
    }
  }
  return null
}

function checkHardcodedSecrets(line: string, file: string, lineNumber: number): Vulnerability | null {
  const secretPatterns = [
    /(?:api[_-]?key|apikey|secret|password|token|auth)\s*[:=]\s*['"][A-Za-z0-9_\-.]{16,}['"]/i,
    /(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{36,}/,
    /sk-[A-Za-z0-9]{32,}/,
    /-----BEGIN (?:RSA |EC )?PRIVATE KEY-----/,
  ]

  for (const pattern of secretPatterns) {
    if (pattern.test(line)) {
      return {
        type: "hardcoded-secret",
        severity: "critical",
        file,
        line: lineNumber,
        message: "Possible hardcoded secret detected",
        recommendation: "Use environment variables or a secrets manager. Never commit secrets to source control.",
      }
    }
  }
  return null
}

function checkCommandInjection(line: string, file: string, lineNumber: number): Vulnerability | null {
  if (/\b(?:exec|spawn|execSync|spawnSync|execFile|execFileSync)\s*\(/.test(line)) {
    return {
      type: "command-injection",
      severity: "critical",
      file,
      line: lineNumber,
      message: "Use of child process execution functions can lead to command injection",
      recommendation: "Avoid passing user input to exec/spawn. Use execFile with argument arrays if needed.",
    }
  }
  return null
}

function checkPrototypePollution(line: string, file: string, lineNumber: number): Vulnerability | null {
  if (/\[\s*['"]__proto__['"]\s*\]/.test(line) || /\.__proto__\s*=/.test(line)) {
    return {
      type: "prototype-pollution",
      severity: "high",
      file,
      line: lineNumber,
      message: "Potential prototype pollution vulnerability",
      recommendation: "Use Object.create(null) or Object.freeze() on prototypes. Validate object keys before assignment.",
    }
  }
  return null
}

async function analyzeDependencies(content: string, file: string): Promise<Vulnerability[]> {
  const vulnerabilities: Vulnerability[] = []

  try {
    const packageJson = JSON.parse(content)
    const allDeps = { ...packageJson.dependencies, ...packageJson.devDependencies } as Record<string, string> | undefined

    if (allDeps) {
      for (const [name, version] of Object.entries(allDeps)) {
        const ver = version.replace(/^[\^~]/, "")
        if (ver === "*") {
          vulnerabilities.push({
            type: "wildcard-dependency",
            severity: "medium",
            file,
            message: `Dependency '${name}' uses wildcard version '${version}'`,
            recommendation: "Pin to a specific version range instead of using '*' to avoid unexpected breaking changes.",
          })
        }
      }
    }
  } catch {
    // Invalid JSON - skip dependency analysis
  }

  return vulnerabilities
}
