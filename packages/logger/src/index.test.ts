import { describe, it, expect, vi, beforeEach } from "vitest"
import { Logger, createLogger, setLogLevel, configureLogger, LogLevel } from "./index.js"

describe("@dash/logger", () => {
  beforeEach(() => {
    configureLogger({ level: LogLevel.Debug, format: "json", destination: "stdout" })
  })

  it("should create a logger with a module name", () => {
    const logger = createLogger("test")
    expect(logger).toBeInstanceOf(Logger)
  })

  it("should support child loggers", () => {
    const logger = createLogger("parent")
    const child = logger.child("child")
    expect(child).toBeInstanceOf(Logger)
  })

  it("should respect log levels", () => {
    const write = vi.spyOn(process.stdout, "write").mockImplementation(() => true)
    configureLogger({ level: LogLevel.Warn })
    const logger = createLogger("test")
    logger.debug("debug message")
    logger.info("info message")
    expect(write).not.toHaveBeenCalled()
    logger.warn("warn message")
    expect(write).toHaveBeenCalled()
    write.mockRestore()
  })

  it("should set global log level", () => {
    setLogLevel(LogLevel.Error)
    const logger = createLogger("test")
    const write = vi.spyOn(process.stdout, "write").mockImplementation(() => true)
    logger.info("should not log")
    expect(write).not.toHaveBeenCalled()
    logger.error("should log")
    expect(write).toHaveBeenCalled()
    write.mockRestore()
  })

  it("should configure format", () => {
    const write = vi.spyOn(process.stdout, "write").mockImplementation(() => true)
    configureLogger({ format: "json" })
    const logger = createLogger("test")
    logger.info("hello")
    const output = JSON.parse(write.mock.calls[0]![0] as string)
    expect(output.message).toBe("hello")
    expect(output.level).toBe(LogLevel.Info)
    expect(output.module).toBe("test")
    write.mockRestore()
  })
})
