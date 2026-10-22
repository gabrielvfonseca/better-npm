import { mkdirSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { afterEach, beforeEach, describe, expect, it } from "vitest"
import { LocalStorage, StorageFactory } from "./index.js"

describe("@better-npm/storage", () => {
  let testDir: string

  beforeEach(() => {
    testDir = join(tmpdir(), `bnpm-storage-test-${Date.now()}`)
    mkdirSync(testDir, { recursive: true })
  })

  afterEach(() => {
    rmSync(testDir, { recursive: true, force: true })
  })

  describe("LocalStorage", () => {
    it("should write and read data", async () => {
      const storage = new LocalStorage(testDir)
      await storage.write("test.txt", "hello world")
      const data = await storage.read("test.txt")
      expect(data.toString()).toBe("hello world")
    })

    it("should check existence", async () => {
      const storage = new LocalStorage(testDir)
      expect(await storage.exists("nonexistent.txt")).toBe(false)
      await storage.write("exists.txt", "data")
      expect(await storage.exists("exists.txt")).toBe(true)
    })

    it("should delete files", async () => {
      const storage = new LocalStorage(testDir)
      await storage.write("temp.txt", "data")
      expect(await storage.exists("temp.txt")).toBe(true)
      await storage.delete("temp.txt")
      expect(await storage.exists("temp.txt")).toBe(false)
    })

    it("should list files with prefix", async () => {
      const storage = new LocalStorage(testDir)
      await storage.write("a/1.txt", "a")
      await storage.write("a/2.txt", "b")
      await storage.write("b/3.txt", "c")
      const files = await storage.list("a")
      expect(files).toHaveLength(2)
    })
  })

  describe("StorageFactory", () => {
    it("should create local storage", () => {
      const storage = StorageFactory.create({ type: "local", baseDir: testDir })
      expect(storage).toBeInstanceOf(LocalStorage)
    })

    it("should throw for unknown type", () => {
      expect(() => StorageFactory.create({ type: "invalid" as never })).toThrow()
    })
  })
})
