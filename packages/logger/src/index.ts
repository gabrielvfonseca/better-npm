export enum LogLevel {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
  Fatal = 4,
}

const LogLevelLabel: Record<LogLevel, string> = {
  [LogLevel.Debug]: "DEBUG",
  [LogLevel.Info]: "INFO",
  [LogLevel.Warn]: "WARN",
  [LogLevel.Error]: "ERROR",
  [LogLevel.Fatal]: "FATAL",
}

export interface LogEntry {
  level: LogLevel
  message: string
  timestamp: string
  module?: string
  requestId?: string
  error?: string
  metadata?: Record<string, unknown>
}

export interface LoggerConfig {
  level?: LogLevel
  format?: "json" | "text"
  destination?: "stdout" | "stderr"
}

let globalConfig: Required<LoggerConfig> = {
  level: LogLevel.Info,
  format: "text",
  destination: "stdout",
}

export class Logger {
  private module: string

  constructor(module: string) {
    this.module = module
  }

  private log(level: LogLevel, message: string, meta?: Record<string, unknown>): void {
    if (level < globalConfig.level) return

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date().toISOString(),
      module: this.module,
      ...(meta?.error ? { error: String(meta.error) } : {}),
      ...(meta ? { metadata: meta } : {}),
    }

    const output = globalConfig.format === "json" ? JSON.stringify(entry) : this.formatText(entry)
    const stream = globalConfig.destination === "stderr" ? process.stderr : process.stdout
    stream.write(output + "\n")
  }

  private formatText(entry: LogEntry): string {
    const level = LogLevelLabel[entry.level].padEnd(5)
    const mod = entry.module ? `[${entry.module}]` : ""
    const err = entry.error ? ` ${entry.error}` : ""
    return `${entry.timestamp} ${level} ${mod} ${entry.message}${err}`
  }

  debug(message: string, meta?: Record<string, unknown>): void {
    this.log(LogLevel.Debug, message, meta)
  }

  info(message: string, meta?: Record<string, unknown>): void {
    this.log(LogLevel.Info, message, meta)
  }

  warn(message: string, meta?: Record<string, unknown>): void {
    this.log(LogLevel.Warn, message, meta)
  }

  error(message: string, meta?: Record<string, unknown>): void {
    this.log(LogLevel.Error, message, meta)
  }

  fatal(message: string, meta?: Record<string, unknown>): void {
    this.log(LogLevel.Fatal, message, meta)
  }

  child(module: string): Logger {
    return new Logger(`${this.module}:${module}`)
  }
}

export function createLogger(module: string): Logger {
  return new Logger(module)
}

export function setLogLevel(level: LogLevel): void {
  globalConfig.level = level
}

export function configureLogger(config: LoggerConfig): void {
  globalConfig = {
    ...globalConfig,
    ...config,
  }
}
