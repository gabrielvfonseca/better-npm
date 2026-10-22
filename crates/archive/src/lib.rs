//! Package archive creation and extraction (tar/zip with optional compression).
//!
//! Provides a unified interface for creating and extracting package
//! archives in various formats.

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::io::{Read, Write};
use std::path::Path;

/// Error type for archive operations.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("archive error: {0}")]
    Archive(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("invalid entry path: {0}")]
    InvalidEntryPath(String),
    #[error("compression error: {0}")]
    Compression(#[from] bnpm_compression::CompressionError),
}

use bnpm_compression::{CompressionAlgo, compress, decompress};

/// Supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ArchiveFormat {
    /// Uncompressed tar archive.
    Tar,
    /// Gzip-compressed tar archive.
    TarGz,
    /// Zip archive.
    Zip,
}

impl ArchiveFormat {
    /// Return the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Tar => ".tar",
            ArchiveFormat::TarGz => ".tar.gz",
            ArchiveFormat::Zip => ".zip",
        }
    }

    /// Detect the format from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.to_string_lossy().to_lowercase();
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Some(ArchiveFormat::TarGz)
        } else if name.ends_with(".tar") {
            Some(ArchiveFormat::Tar)
        } else if name.ends_with(".zip") {
            Some(ArchiveFormat::Zip)
        } else {
            None
        }
    }
}

/// A single entry within an archive.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Path within the archive.
    pub path: String,
    /// File contents.
    pub data: Vec<u8>,
    /// File mode (Unix permissions).
    pub mode: Option<u32>,
    /// File size in bytes.
    pub size: u64,
}

impl ArchiveEntry {
    /// Create a new archive entry.
    pub fn new(path: impl Into<String>, data: Vec<u8>) -> Self {
        let size = data.len() as u64;
        Self {
            path: path.into(),
            data,
            mode: None,
            size,
        }
    }

    /// Set the file mode.
    pub fn with_mode(mut self, mode: u32) -> Self {
        self.mode = Some(mode);
        self
    }
}

/// Create an archive from a source directory.
pub fn create_archive(source: &Path, format: ArchiveFormat) -> Result<Vec<u8>, ArchiveError> {
    match format {
        ArchiveFormat::Tar => create_tar(source, false),
        ArchiveFormat::TarGz => create_tar(source, true),
        ArchiveFormat::Zip => create_zip(source),
    }
}

fn create_tar(source: &Path, gzip: bool) -> Result<Vec<u8>, ArchiveError> {
    let mut tar_bytes = Vec::new();

    {
        let mut builder = tar::Builder::new(&mut tar_bytes);

        // Walk the source directory and add files
        let walker = walkdir::WalkDir::new(source);
        for entry in walker.into_iter() {
            let entry = entry?;
            let path = entry.path();

            if path == source {
                continue; // Skip the root directory itself
            }

            let relative = path
                .strip_prefix(source)
                .map_err(|e| ArchiveError::Archive(e.to_string()))?;

            if entry.file_type().is_dir() {
                    let header = tar::Header::new_gnu();
                    builder
                        .append_dir(relative.to_string_lossy().as_ref(), path)
                        .map_err(|e| ArchiveError::Archive(e.to_string()))?;
            } else if entry.file_type().is_file() {
                let mut header = tar::Header::new_gnu();
                let metadata = entry.metadata()?;
                header.set_size(metadata.len());
                header.set_mtime(
                    metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                );
                header.set_mode(metadata.permissions().mode());

                let file_data = std::fs::read(path)?;
                builder
                    .append_data(&mut header, relative.to_string_lossy().as_ref(), file_data.as_slice())
                    .map_err(|e| ArchiveError::Archive(e.to_string()))?;
            }
        }

        builder
            .finish()
            .map_err(|e| ArchiveError::Archive(e.to_string()))?;
    }

    if gzip {
        let compressed = compress(&tar_bytes, CompressionAlgo::Gzip)?;
        Ok(compressed)
    } else {
        Ok(tar_bytes)
    }
}

fn create_zip(source: &Path) -> Result<Vec<u8>, ArchiveError> {
    let mut zip_bytes = Vec::new();
    let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));

    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let walker = walkdir::WalkDir::new(source);
    for entry in walker.into_iter() {
        let entry = entry?;
        let path = entry.path();

        if path == source {
            continue;
        }

        let relative = path
            .strip_prefix(source)
            .map_err(|e| ArchiveError::Archive(e.to_string()))?;

        let name = relative.to_string_lossy().to_string();

        if entry.file_type().is_dir() {
            zip_writer
                .add_directory(&name, options)
                .map_err(|e| ArchiveError::Archive(e.to_string()))?;
        } else if entry.file_type().is_file() {
            zip_writer
                .start_file(&name, options)
                .map_err(|e| ArchiveError::Archive(e.to_string()))?;
            let data = std::fs::read(path)?;
            zip_writer
                .write_all(&data)
                .map_err(|e| ArchiveError::Archive(e.to_string()))?;
        }
    }

    zip_writer
        .finish()
        .map_err(|e| ArchiveError::Archive(e.to_string()))?;

    Ok(zip_bytes)
}

/// Extract an archive into a destination directory.
pub fn extract_archive(data: &[u8], dest: &Path, format: ArchiveFormat) -> Result<Vec<ArchiveEntry>, ArchiveError> {
    match format {
        ArchiveFormat::Tar => extract_tar(data, dest, false),
        ArchiveFormat::TarGz => extract_tar(data, dest, true),
        ArchiveFormat::Zip => extract_zip(data, dest),
    }
}

fn extract_tar(data: &[u8], dest: &Path, gzipped: bool) -> Result<Vec<ArchiveEntry>, ArchiveError> {
    let data = if gzipped {
        decompress(data, CompressionAlgo::Gzip)?
    } else {
        data.to_vec()
    };

    let mut archive = tar::Archive::new(&data[..]);
    let mut entries = Vec::new();

    std::fs::create_dir_all(dest)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        // Prevent path traversal
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(ArchiveError::InvalidEntryPath(path.to_string_lossy().to_string()));
        }

        let dest_path = dest.join(&path);

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;

            std::fs::write(&dest_path, &data)?;

            entries.push(ArchiveEntry {
                path: path.to_string_lossy().to_string(),
                data,
                mode: Some(entry.header().mode()?),
                size: entry.header().size()?,
            });
        }
    }

    Ok(entries)
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<Vec<ArchiveEntry>, ArchiveError> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|e| ArchiveError::Archive(e.to_string()))?;

    let mut entries = Vec::new();

    std::fs::create_dir_all(dest)?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ArchiveError::Archive(e.to_string()))?;

        let path = file
            .name()
            .trim_start_matches('/')
            .to_string();

        // Prevent path traversal
        if path.contains("..") {
            continue;
        }

        let dest_path = dest.join(&path);

        if file.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            std::fs::write(&dest_path, &data)?;

            let size = data.len() as u64;
            entries.push(ArchiveEntry {
                path,
                data,
                mode: None,
                size,
            });
        }
    }

    Ok(entries)
}

/// List the entries in an archive without extracting.
pub fn list_archive(data: &[u8], format: ArchiveFormat) -> Result<Vec<String>, ArchiveError> {
    match format {
        ArchiveFormat::Tar | ArchiveFormat::TarGz => {
            let data = if format == ArchiveFormat::TarGz {
                decompress(data, CompressionAlgo::Gzip)?
            } else {
                data.to_vec()
            };
    let mut archive = tar::Archive::new(&data[..]);
    let entries: Vec<String> = archive
                .entries()?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
                .collect();
            Ok(entries)
        }
        ArchiveFormat::Zip => {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
                .map_err(|e| ArchiveError::Archive(e.to_string()))?;
            let len = archive.len();
            let mut entries = Vec::with_capacity(len);
            for i in 0..len {
                if let Ok(file) = archive.by_index(i) {
                    entries.push(file.name().to_string());
                }
            }
            Ok(entries)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_extract_tar() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("file.txt"), b"hello from tar").unwrap();
        fs::write(src.join("nested.txt"), b"nested file").unwrap();

        let archive = create_archive(&src, ArchiveFormat::Tar).unwrap();
        assert!(!archive.is_empty());

        let dest = tempfile::tempdir().unwrap();
        let entries = extract_archive(&archive, dest.path(), ArchiveFormat::Tar).unwrap();
        assert_eq!(entries.len(), 2);

        assert!(dest.path().join("file.txt").exists());
        assert!(dest.path().join("nested.txt").exists());
    }

    #[test]
    fn test_create_extract_tar_gz() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("data.bin"), b"compressed tar data").unwrap();

        let archive = create_archive(&src, ArchiveFormat::TarGz).unwrap();
        assert!(!archive.is_empty());

        let dest = tempfile::tempdir().unwrap();
        let entries = extract_archive(&archive, dest.path(), ArchiveFormat::TarGz).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(dest.path().join("data.bin").exists());
    }

    #[test]
    fn test_create_extract_zip() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("doc.txt"), b"zip content").unwrap();

        let archive = create_archive(&src, ArchiveFormat::Zip).unwrap();
        assert!(!archive.is_empty());

        let dest = tempfile::tempdir().unwrap();
        let entries = extract_archive(&archive, dest.path(), ArchiveFormat::Zip).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(dest.path().join("doc.txt").exists());
    }

    #[test]
    fn test_list_archive() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("listme.txt"), b"list this").unwrap();

        let archive = create_archive(&src, ArchiveFormat::Tar).unwrap();
        let names = list_archive(&archive, ArchiveFormat::Tar).unwrap();
        assert!(names.iter().any(|n| n.contains("listme.txt")));
    }

    #[test]
    fn test_archive_entry_new() {
        let entry = ArchiveEntry::new("path/to/file.txt", b"content".to_vec());
        assert_eq!(entry.size, 7);
        assert_eq!(entry.path, "path/to/file.txt");
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            ArchiveFormat::from_path(Path::new("pkg.tar.gz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("pkg.tgz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("pkg.tar")),
            Some(ArchiveFormat::Tar)
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("pkg.zip")),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(ArchiveFormat::from_path(Path::new("pkg.txt")), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(ArchiveFormat::Tar.extension(), ".tar");
        assert_eq!(ArchiveFormat::TarGz.extension(), ".tar.gz");
        assert_eq!(ArchiveFormat::Zip.extension(), ".zip");
    }
}
