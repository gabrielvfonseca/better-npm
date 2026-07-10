//! Symbolic and hard linking of installed packages into target directories.
//!
//! Manages the creation and removal of binary symlinks,
//! node_modules/.bin entries, and other system-level links.

use std::path::{Path, PathBuf};

/// Error type for linker operations.
#[derive(Debug, thiserror::Error)]
pub enum LinkerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("binary not found: {0}")]
    BinaryNotFound(String),
    #[error("package not found: {0}")]
    PackageNotFound(String),
    #[error("target already exists: {0}")]
    TargetExists(PathBuf),
    #[error("link creation failed: {0}")]
    LinkFailed(String),
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
}

/// A linked binary with its source and target paths.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkedBinary {
    /// Name of the binary (e.g., "tsc", "eslint").
    pub name: String,
    /// Full path to the linked binary.
    pub path: PathBuf,
    /// Target package name.
    pub target: String,
    /// The kind of link that was created.
    pub link_type: LinkType,
}

/// The type of link created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LinkType {
    /// Symbolic link.
    Symlink,
    /// Hard link.
    HardLink,
    /// Copy (fallback when linking fails).
    Copy,
}

/// Configuration for the linker.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkerConfig {
    /// Directory where binaries should be linked.
    pub bin_dir: PathBuf,
    /// Preferred link method.
    pub link_type: LinkType,
    /// Whether to overwrite existing links.
    pub overwrite: bool,
    /// Whether to create parent directories.
    pub create_parents: bool,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            bin_dir: PathBuf::from("node_modules/.bin"),
            link_type: LinkType::Symlink,
            overwrite: false,
            create_parents: true,
        }
    }
}

/// The linker manages creation and removal of binary links.
pub struct Linker {
    store: std::sync::Arc<dyn dash_store::StoreBackend>,
    fs: std::sync::Arc<dyn dash_filesystem::FsOps>,
    config: LinkerConfig,
}

// Simple FsOps trait for testability
mod dash_filesystem {
    use std::path::Path;

    #[async_trait::async_trait]
    pub trait FsOps: Send + Sync {
        async fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error>;
        async fn hard_link(&self, original: &Path, link: &Path) -> Result<(), std::io::Error>;
        async fn copy(&self, from: &Path, to: &Path) -> Result<(), std::io::Error>;
        async fn remove(&self, path: &Path) -> Result<(), std::io::Error>;
        async fn exists(&self, path: &Path) -> bool;
        async fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error>;
    }

    pub struct RealFs;

    #[async_trait::async_trait]
    impl FsOps for RealFs {
        async fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(original, link)?;
            }
            #[cfg(not(unix))]
            {
                std::os::windows::fs::symlink_file(original, link)?;
            }
            Ok(())
        }

        async fn hard_link(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
            std::fs::hard_link(original, link)
        }

        async fn copy(&self, from: &Path, to: &Path) -> Result<(), std::io::Error> {
            std::fs::copy(from, to)?;
            Ok(())
        }

        async fn remove(&self, path: &Path) -> Result<(), std::io::Error> {
            if path.is_symlink() || path.is_file() {
                std::fs::remove_file(path)?;
            } else if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            }
            Ok(())
        }

        async fn exists(&self, path: &Path) -> bool {
            path.exists()
        }

        async fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
            std::fs::create_dir_all(path)
        }
    }
}

impl Linker {
    /// Create a new linker.
    pub fn new(
        store: std::sync::Arc<dyn dash_store::StoreBackend>,
        config: LinkerConfig,
    ) -> Self {
        Self {
            store,
            fs: std::sync::Arc::new(dash_filesystem::RealFs),
            config,
        }
    }

    /// Create a new linker with a custom filesystem abstraction (for testing).
    pub fn new_with_fs(
        store: std::sync::Arc<dyn dash_store::StoreBackend>,
        fs: std::sync::Arc<dyn dash_filesystem::FsOps>,
        config: LinkerConfig,
    ) -> Self {
        Self { store, fs, config }
    }

    /// Link binaries for a package into the bin directory.
    pub async fn link_binaries(&self, pkg: &dash_package::Package) -> Result<Vec<LinkedBinary>, LinkerError> {
        let bin_map = self.extract_binaries(pkg)?;
        if bin_map.is_empty() {
            return Ok(Vec::new());
        }

        // Ensure the bin directory exists
        if self.config.create_parents {
            self.fs
                .create_dir_all(&self.config.bin_dir)
                .await?;
        }

        let mut linked = Vec::new();

        for (binary_name, source_path) in &bin_map {
            let link_path = self.config.bin_dir.join(binary_name);

            // Resolve the source path relative to the package
            let source = PathBuf::from(source_path);
            let actual_source = if source.is_absolute() {
                source
            } else {
                // Relative to the package root
                PathBuf::from(".").join(&source) // simplified
            };

            // Check for existing link
            if self.fs.exists(&link_path).await {
                if !self.config.overwrite {
                    return Err(LinkerError::TargetExists(link_path));
                }
                self.fs.remove(&link_path).await?;
            }

            // Create the link
            match self.config.link_type {
                LinkType::Symlink => {
                    self.fs.symlink(&actual_source, &link_path).await?;
                }
                LinkType::HardLink => {
                    self.fs.hard_link(&actual_source, &link_path).await?;
                }
                LinkType::Copy => {
                    self.fs.copy(&actual_source, &link_path).await?;
                }
            }

            linked.push(LinkedBinary {
                name: binary_name.clone(),
                path: link_path,
                target: pkg.name.to_string(),
                link_type: self.config.link_type,
            });
        }

        Ok(linked)
    }

    /// Unlink binaries for a package.
    pub async fn unlink_binaries(&self, name: &dash_package::PackageName) -> Result<(), LinkerError> {
        let bin_dir = &self.config.bin_dir;

        if !bin_dir.exists() {
            return Ok(());
        }

        let mut read_dir = tokio::fs::read_dir(bin_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Check if this link points to the package being unlinked
            if path.is_symlink() || path.is_file() {
                let target = tokio::fs::read_link(&path).await;
                if let Ok(target_path) = target {
                    if target_path.to_string_lossy().contains(name.as_str()) {
                        tokio::fs::remove_file(&path).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Unlink a specific binary by name.
    pub async fn unlink_binary(&self, name: &str) -> Result<(), LinkerError> {
        let link_path = self.config.bin_dir.join(name);
        if link_path.exists() {
            if self.config.create_parents {
                self.fs.create_dir_all(&self.config.bin_dir).await?;
            }
            self.fs.remove(&link_path).await?;
        }
        Ok(())
    }

    /// List all currently linked binaries.
    pub async fn list_linked(&self) -> Result<Vec<LinkedBinary>, LinkerError> {
        let bin_dir = &self.config.bin_dir;
        if !bin_dir.exists() {
            return Ok(Vec::new());
        }

        let mut linked = Vec::new();
        let mut read_dir = tokio::fs::read_dir(bin_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_symlink() || path.is_file() {
                if let Ok(target) = tokio::fs::read_link(&path).await {
                    let target_str = target.to_string_lossy().to_string();
                    linked.push(LinkedBinary {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path,
                        target: target_str,
                        link_type: LinkType::Symlink,
                    });
                }
            }
        }

        Ok(linked)
    }

    fn extract_binaries(&self, pkg: &dash_package::Package) -> Result<std::collections::HashMap<String, String>, LinkerError> {
        let mut bins = std::collections::HashMap::new();

        // Parse the manifest for "bin" entries
        if let Some(bin) = pkg.manifest.get("bin") {
            match bin {
                serde_json::Value::String(s) => {
                    // Single binary: use package name
                    bins.insert(pkg.name.unescoped_name().to_string(), s.clone());
                }
                serde_json::Value::Object(map) => {
                    for (name, path) in map {
                        if let Some(path_str) = path.as_str() {
                            bins.insert(name.clone(), path_str.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(bins)
    }

    /// Get the linker configuration.
    pub fn config(&self) -> &LinkerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_package::{Package, PackageName};
    use std::sync::Arc;

    struct MockFs {
        created_symlinks: std::sync::Mutex<Vec<(PathBuf, PathBuf)>>,
        created_dirs: std::sync::Mutex<Vec<PathBuf>>,
    }

    impl MockFs {
        fn new() -> Self {
            Self {
                created_symlinks: std::sync::Mutex::new(Vec::new()),
                created_dirs: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl dash_filesystem::FsOps for MockFs {
        async fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
            self.created_symlinks.lock().unwrap().push((original.to_path_buf(), link.to_path_buf()));
            Ok(())
        }

        async fn hard_link(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
            self.created_symlinks.lock().unwrap().push((original.to_path_buf(), link.to_path_buf()));
            Ok(())
        }

        async fn copy(&self, from: &Path, to: &Path) -> Result<(), std::io::Error> {
            Ok(())
        }

        async fn remove(&self, _path: &Path) -> Result<(), std::io::Error> {
            Ok(())
        }

        async fn exists(&self, path: &Path) -> bool {
            // Pretend nothing exists yet
            false
        }

        async fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
            self.created_dirs.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_link_binaries_single() {
        let store = Arc::new(dash_store::LocalStore::new(PathBuf::from("/tmp/test-store")));
        let mock_fs = Arc::new(MockFs::new());

        let config = LinkerConfig {
            bin_dir: PathBuf::from("/tmp/test-bin"),
            link_type: LinkType::Symlink,
            overwrite: false,
            create_parents: true,
        };

        let linker = Linker::new_with_fs(store, mock_fs.clone(), config);

        let name = PackageName::new_unchecked("test-pkg");
        let version = semver::Version::new(1, 0, 0);
        let mut pkg = Package::new(name, version);
        pkg.manifest = serde_json::json!({
            "bin": "./bin/cli.js"
        });

        let linked = linker.link_binaries(&pkg).await.unwrap();
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0].name, "test-pkg");
    }

    #[tokio::test]
    async fn test_link_binaries_multi() {
        let store = Arc::new(dash_store::LocalStore::new(PathBuf::from("/tmp/test-store")));
        let mock_fs = Arc::new(MockFs::new());

        let config = LinkerConfig {
            bin_dir: PathBuf::from("/tmp/test-bin"),
            link_type: LinkType::Symlink,
            overwrite: false,
            create_parents: true,
        };

        let linker = Linker::new_with_fs(store, mock_fs.clone(), config);

        let name = PackageName::new_unchecked("multi-bin-pkg");
        let version = semver::Version::new(1, 0, 0);
        let mut pkg = Package::new(name, version);
        pkg.manifest = serde_json::json!({
            "bin": {
                "cli-one": "./bin/cli-one.js",
                "cli-two": "./bin/cli-two.js"
            }
        });

        let linked = linker.link_binaries(&pkg).await.unwrap();
        assert_eq!(linked.len(), 2);
    }

    #[tokio::test]
    async fn test_link_binaries_no_bin() {
        let store = Arc::new(dash_store::LocalStore::new(PathBuf::from("/tmp/test-store")));
        let mock_fs = Arc::new(MockFs::new());

        let config = LinkerConfig::default();
        let linker = Linker::new_with_fs(store, mock_fs, config);

        let name = PackageName::new_unchecked("no-bin");
        let version = semver::Version::new(1, 0, 0);
        let pkg = Package::new(name, version);

        let linked = linker.link_binaries(&pkg).await.unwrap();
        assert!(linked.is_empty());
    }

    #[tokio::test]
    async fn test_unlink_binary() {
        let store = Arc::new(dash_store::LocalStore::new(PathBuf::from("/tmp/test-store")));
        let mock_fs = Arc::new(MockFs::new());

        let config = LinkerConfig::default();
        let linker = Linker::new_with_fs(store, mock_fs, config);

        // Should not error when unlinking a non-existent binary
        let result = linker.unlink_binary("non-existent").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_linker_config_default() {
        let config = LinkerConfig::default();
        assert_eq!(config.bin_dir, PathBuf::from("node_modules/.bin"));
        assert!(!config.overwrite);
        assert!(config.create_parents);
    }

    #[test]
    fn test_linked_binary() {
        let lb = LinkedBinary {
            name: "tsc".to_string(),
            path: PathBuf::from("/usr/local/bin/tsc"),
            target: "typescript".to_string(),
            link_type: LinkType::Symlink,
        };
        assert_eq!(lb.name, "tsc");
    }
}
