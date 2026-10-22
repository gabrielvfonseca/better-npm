# `@better-npm/hooks`

React hooks for common UI patterns.

## Overview

Collection of React 19 hooks: `useLocalStorage`, `useDebounce`, `useThrottle`, `useMediaQuery`, `usePrevious`, `useClipboard`, `useInterval`, `useIntersectionObserver`.

## Usage

```tsx
import { useDebounce, useClipboard, useMediaQuery } from "@better-npm/hooks";

const debounced = useDebounce(value, 300);
const [copied, copy] = useClipboard();
const isMobile = useMediaQuery("(max-width: 768px)");
```
