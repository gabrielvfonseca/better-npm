# `@better-npm/shared`

Shared utility functions and event emitter.

## Overview

Collection of general-purpose utilities: `sleep()`, `randomId()`, `omit()`, `pick()`, `deepMerge()`, `retry()` (exponential backoff), `timeout()`, `chunk()`, `debounce()`, `throttle()`, and an `EventEmitter` class.

## Usage

```ts
import { retry, deepMerge, EventEmitter } from "@better-npm/shared";

const result = await retry(() => fetchData(), { retries: 3, delay: 1000 });
const merged = deepMerge({ a: 1 }, { b: 2 });

const emitter = new EventEmitter<{ ready: [] }>();
emitter.on("ready", () => console.log("ready"));
emitter.emit("ready");
```
