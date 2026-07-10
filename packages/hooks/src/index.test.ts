import { describe, it, expect } from "vitest"
import {
  useDebounce,
  useThrottle,
  useMediaQuery,
  usePrevious,
  useLocalStorage,
  useClipboard,
  useInterval,
  useIntersectionObserver,
} from "./index.js"

describe("@dash/hooks", () => {
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
