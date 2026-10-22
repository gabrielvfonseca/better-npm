# bnpm-wasm

WebAssembly runtime integration for running WASM-based packages.

## Overview

Provides compilation, validation, and execution of WebAssembly modules within the Bnpm environment.

## Features

- WASM module compilation
- Module validation and security checks
- WASM execution sandboxing
- Integration with `bnpm-runtime` lifecycle
- Bytecode caching for fast startup

## Usage

```rust
use bnpm_wasm::WasmRuntime;

let runtime = WasmRuntime::new(&config)?;
let instance = runtime.instantiate(wasm_bytes)?;
let result = instance.call("main", &args)?;
```
