# dash-filesystem

Filesystem abstractions: path resolution, locking, atomic writes, and directory walks.

## Overview

Safe, cross-platform filesystem operations for Dash including atomic file writes, directory walking, path resolution, and file locking.

## Features

- Atomic file read/write operations
- Directory walking and glob matching
- Cross-platform path resolution
- File locking for concurrent safety
- Temporary file/directory creation

## Usage

```rust
use dash_filesystem::{atomic_write, lock_file, walk_directory};

atomic_write("path/to/file.txt", b"content")?;
let guard = lock_file("path/to/lock.lock")?;
```
