import { describe, expect, it } from "vitest"
import {
  useClipboard,
  useDebounce,
  useIntersectionObserver,
  useInterval,
  useLocalStorage,
  useMediaQuery,
  usePrevious,
  useThrottle,
} from "./index.js"

describe("@better-npm/hooks", () => {
  it("should export all hooks", () => {
    expect(useDebounce).toBeDefined()
    expect(useThrottle).toBeDefined()
    expect(useMediaQuery).toBeDefined()
    expect(usePrevious).toBeDefined()
    expect(useLocalStorage).toBeDefined()
    expect(useClipboard).toBeDefined()
    expect(useInterval).toBeDefined()
    expect(useIntersectionObserver).toBeDefined()
  })

  it("should be functions", () => {
    expect(typeof useDebounce).toBe("function")
    expect(typeof useThrottle).toBe("function")
    expect(typeof useMediaQuery).toBe("function")
    expect(typeof useClipboard).toBe("function")
  })
})
