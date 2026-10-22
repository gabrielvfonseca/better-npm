import { type RefObject, useCallback, useEffect, useRef, useState } from "react"

export function useLocalStorage<T>(key: string, initialValue: T): [T, (value: T) => void] {
  const [storedValue, setStoredValue] = useState<T>(() => {
    try {
      const item = globalThis.localStorage?.getItem(key)
      return item ? (JSON.parse(item) as T) : initialValue
    } catch {
      return initialValue
    }
  })

  const setValue = useCallback(
    (value: T) => {
      setStoredValue(value)
      try {
        globalThis.localStorage?.setItem(key, JSON.stringify(value))
      } catch {
        // quota exceeded or private browsing
      }
    },
    [key],
  )

  return [storedValue, setValue]
}

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay)
    return () => clearTimeout(timer)
  }, [value, delay])

  return debouncedValue
}

export function useThrottle<T>(value: T, delay: number): T {
  const [throttledValue, setThrottledValue] = useState<T>(value)
  const lastRun = useRef(Date.now())

  useEffect(() => {
    const now = Date.now()
    const remaining = delay - (now - lastRun.current)

    if (remaining <= 0) {
      lastRun.current = now
      setThrottledValue(value)
      return
    }

    const timer = setTimeout(() => {
      lastRun.current = Date.now()
      setThrottledValue(value)
    }, remaining)

    return () => clearTimeout(timer)
  }, [value, delay])

  return throttledValue
}

export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(false)

  useEffect(() => {
    const mediaQuery = globalThis.matchMedia(query)
    setMatches(mediaQuery.matches)

    const handler = (event: MediaQueryListEvent) => setMatches(event.matches)
    mediaQuery.addEventListener("change", handler)
    return () => mediaQuery.removeEventListener("change", handler)
  }, [query])

  return matches
}

export function usePrevious<T>(value: T): T | undefined {
  const ref = useRef<T | undefined>(undefined)

  useEffect(() => {
    ref.current = value
  })

  return ref.current
}

export function useClipboard(): { copy: (text: string) => Promise<void>; copied: boolean } {
  const [copied, setCopied] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const copy = useCallback(async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      if (timeoutRef.current) clearTimeout(timeoutRef.current)
      timeoutRef.current = setTimeout(() => setCopied(false), 2000)
    } catch {
      setCopied(false)
    }
  }, [])

  return { copy, copied }
}

export function useInterval(callback: () => void, delay: number | null): void {
  const savedCallback = useRef(callback)

  useEffect(() => {
    savedCallback.current = callback
  }, [callback])

  useEffect(() => {
    if (delay === null) return

    const id = setInterval(() => savedCallback.current(), delay)
    return () => clearInterval(id)
  }, [delay])
}

export function useIntersectionObserver(
  ref: RefObject<Element | null>,
  options?: IntersectionObserverInit,
): IntersectionObserverEntry | null {
  const [entry, setEntry] = useState<IntersectionObserverEntry | null>(null)

  useEffect(() => {
    const element = ref.current
    if (!element) return

    const observer = new IntersectionObserver(([e]) => {
      if (e) setEntry(e)
    }, options)

    observer.observe(element)
    return () => observer.disconnect()
  }, [ref, options])

  return entry
}
