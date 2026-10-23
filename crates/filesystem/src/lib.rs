//! Filesystem abstractions: path resolution, locking, atomic writes, and directory walks.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Error type for filesystem operations.
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("path not found: {0}")]
    NotFound(PathBuf),
    #[error("path already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    #[error("atomic write failed: {0}")]
    AtomicWrite(String),
}

/// Ensure a directory exists, creating it and all parent directories as needed.
pub fn ensure_dir(path: &Path) -> Result<(), FsError> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Remove a directory and all its contents.
pub fn remove_dir(path: &Path) -> Result<(), FsError> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Copy a directory tree from `src` to `dst`.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<(), FsError> {
    ensure_dir(dst)?;

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            ensure_dir(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }

    Ok(())
}

/// Read a file's contents into a string.
pub fn read_to_string(path: &Path) -> Result<String, FsError> {
    Ok(std::fs::read_to_string(path)?)
}

/// Write bytes to a file (overwriting if it exists).
pub fn write(path: &Path, content: &[u8]) -> Result<(), FsError> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    Ok(std::fs::write(path, content)?)
}

/// Atomically write bytes to a file using a temporary file and rename.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), FsError> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let tmp_path = {
        let uuid = uuid::Uuid::new_v4();
        let mut p = path.to_path_buf();
        let ext = format!(".tmp.{}", uuid);
        let file_name = path
            .file_name()
            .map(|n| format!("{}{}", n.to_string_lossy(), ext))
            .unwrap_or_else(|| uuid.to_string());
        p.set_file_name(file_name);
        p
    };

    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

/// An RAII guard that removes a temporary directory on drop.
pub struct TempDir {
    path: PathBuf,
    released: Arc<AtomicBool>,
}

impl TempDir {
    /// Create a new temporary directory.
    pub fn new() -> Result<Self, FsError> {
        let dir = tempfile::Builder::new()
            .prefix("bnpm-")
            .tempdir()
            .map_err(FsError::Io)?;
        let path = dir.keep();
        Ok(Self {
            path,
            released: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Create a temporary directory with a given prefix.
    pub fn with_prefix(prefix: &str) -> Result<Self, FsError> {
        let dir = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .map_err(FsError::Io)?;
        let path = dir.keep();
        Ok(Self {
            path,
            released: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the path of the temporary directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Release the temporary directory (prevent cleanup on drop).
    pub fn release(self) -> PathBuf {
        self.released.store(true, Ordering::SeqCst);
        self.path.clone()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !self.released.load(Ordering::SeqCst) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// Walk a directory recursively, yielding paths to all files.
pub fn walk_dir(path: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
}

/// Read a file into bytes.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, FsError> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Check if a path exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Get the size of a file in bytes.
pub fn file_size(path: &Path) -> Result<u64, FsError> {
    Ok(std::fs::metadata(path)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_dir() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("a").join("b").join("c");
        ensure_dir(&sub).unwrap();
        assert!(sub.exists());
    }

    #[test]
    fn test_write_and_read() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.txt");
        write(&file, b"hello world").unwrap();
        let content = read_to_string(&file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_atomic_write() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("atomic.txt");
        atomic_write(&file, b"atomic content").unwrap();
        let content = read_to_string(&file).unwrap();
        assert_eq!(content, "atomic content");
    }

    #[test]
    fn test_temp_dir_cleanup() {
        let path;
        {
            let tmp = TempDir::new().unwrap();
            path = tmp.path().to_path_buf();
            assert!(path.exists());
        }
        assert!(!path.exists());
    }

    #[test]
    fn test_temp_dir_release() {
        let path;
        {
            let tmp = TempDir::new().unwrap();
            path = tmp.release();
            assert!(path.exists());
        }
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_walk_dir() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.txt"), b"a").unwrap();
        write(&tmp.path().join("b.txt"), b"b").unwrap();
        let files: Vec<_> = walk_dir(tmp.path()).collect();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_copy_dir() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        write(&src.path().join("file.txt"), b"content").unwrap();
        copy_dir(src.path(), dst.path()).unwrap();
        assert!(dst.path().join("file.txt").exists());
    }

    #[test]
    fn test_remove_dir() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("to_remove.txt");
        write(&file, b"x").unwrap();
        assert!(file.exists());
        remove_dir(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_file_size() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("size.txt");
        write(&file, b"12345").unwrap();
        assert_eq!(file_size(&file).unwrap(), 5);
    }
}
