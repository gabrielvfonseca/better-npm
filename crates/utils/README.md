# bnpm-utils

General utility functions for Bnpm.

## Overview

Includes version comparison, glob matching, string formatting, slug generation, and byte/duration formatting utilities.

## Features

- String formatting and slug generation
- Byte size formatting (human-readable)
- Duration formatting
- Glob pattern matching
- Version string comparison helpers

## Usage

```rust
use bnpm_utils::{format_bytes, slugify, glob_match};

println!("{}", format_bytes(1_500_000)); // "1.43 MB"
println!("{}", slugify("Hello World")); // "hello-world"
assert!(glob_match("*.rs", "lib.rs"));
```
