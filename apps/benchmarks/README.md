# `@better-npm/benchmarks`

Performance benchmark suite for Bnpm subsystems.

## Overview

Pre-defined benchmarks covering crypto hashing, JSON parsing, array sorting, string concatenation, regex matching, set/Map lookups, Promise.all, array reduce, and UUID generation. Supports running individual or all benchmarks with summary output.

## Usage

```ts
import { listBenchmarks, runBenchmark, runBenchmarks } from "@better-npm/benchmarks";

console.log(listBenchmarks());
const result = await runBenchmark("crypto-hash");
const all = await runBenchmarks({ outputFile: "benchmark-results.json" });
```
