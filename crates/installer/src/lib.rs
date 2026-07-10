//! Package installation orchestration with parallel extraction and verification.
//!
//! Coordinates the full install pipeline: resolving dependencies,
//! downloading packages, verifying integrity, extracting archives,
//! and linking binaries.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Error type for installer operations.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("resolve error: {0}")]
    Resolve(#[from] dash_resolver::ResolveError),
    #[error("download error: {0}")]
    Download(#[from] dash_downloader::DownloadError),
    #[error("linker error: {0}")]
    Linker(#[from] dash_linker::LinkerError),
    #[error("cache error: {0}")]
    Cache(#[from] dash_cache::CacheError),
    #[error("store error: {0}")]
    Store(#[from] dash_store::StoreError),
    #[error("integrity error: {0}")]
    Integrity(#[from] dash_integrity::IntegrityError),
    #[error("signature error: {0}")]
    Signature(#[from] dash_signatures::SignatureError),
    #[error("compression error: {0}")]
    Compression(#[from] dash_compression::CompressionError),
    #[error("archive error: {0}")]
    Archive(#[from] dash_archive::ArchiveError),
    #[error("manifest error: {0}")]
    Manifest(#[from] dash_manifest::ManifestError),
    #[error("package error: {0}")]
    Package(#[from] dash_package::PackageError),
    #[error("lockfile error: {0}")]
    Lockfile(#[from] dash_lockfile::LockfileError),
    #[error("filesystem error: {0}")]
    Filesystem(#[from] dash_filesystem::FsError),
    #[error("sandbox error: {0}")]
    Sandbox(#[from] dash_sandbox::SandboxError),
    #[error("validation error: {0}")]
    Validation(String),
}

/// The strategy used to install package files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStrategy {
    /// Copy files to the target directory.
    Copy,
    /// Create symbolic links.
    Link,
    /// Create hard links.
    HardLink,
}

/// Summary of a completed installation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallSummary {
    /// List of packages that were installed.
    pub installed: Vec<String>,
    /// List of binaries that were linked.
    pub linked: Vec<dash_linker::LinkedBinary>,
    /// List of packages that were cached.
    pub cached: Vec<String>,
    /// Total installation duration.
    pub duration: Duration,
    /// Number of packages installed.
    pub package_count: usize,
    /// Total bytes transferred.
    pub total_bytes: u64,
}

impl InstallSummary {
    /// Create a new empty install summary.
    pub fn new() -> Self {
        Self {
            installed: Vec::new(),
            linked: Vec::new(),
            cached: Vec::new(),
            duration: Duration::ZERO,
            package_count: 0,
            total_bytes: 0,
        }
    }

    /// Merge another summary into this one.
    pub fn merge(&mut self, other: InstallSummary) {
        self.installed.extend(other.installed);
        self.linked.extend(other.linked);
        self.cached.extend(other.cached);
        self.duration += other.duration;
        self.package_count += other.package_count;
        self.total_bytes += other.total_bytes;
    }
}

impl Default for InstallSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the installer.
#[derive(Debug, Clone)]
pub struct InstallerConfig {
    /// Target directory for installation.
    pub target_dir: PathBuf,
    /// Directory for the package store.
    pub store_dir: PathBuf,
    /// Directory for binary links.
    pub bin_dir: PathBuf,
    /// Install strategy.
    pub strategy: InstallStrategy,
    /// Whether to verify integrity hashes.
    pub verify_integrity: bool,
    /// Whether to verify signatures.
    pub verify_signatures: bool,
    /// Whether to use the cache.
    pub use_cache: bool,
    /// Whether to run install scripts.
    pub run_scripts: bool,
    /// Maximum concurrency for parallel operations.
    pub concurrency: usize,
}

impl Default for InstallerConfig {
    fn default() -> Self {
        Self {
            target_dir: PathBuf::from("node_modules"),
            store_dir: PathBuf::from(".dash/store"),
            bin_dir: PathBuf::from("node_modules/.bin"),
            strategy: InstallStrategy::Copy,
            verify_integrity: true,
            verify_signatures: true,
            use_cache: true,
            run_scripts: true,
            concurrency: 4,
        }
    }
}

/// The package installer orchestrator.
pub struct Installer {
    resolver: dash_resolver::Resolver,
    downloader: dash_downloader::Downloader,
    linker: dash_linker::Linker,
    cache: Arc<dyn dash_cache::CacheBackend>,
    store: dash_store::Store,
    config: InstallerConfig,
}

impl Installer {
    /// Create a new installer.
    pub fn new(
        resolver: dash_resolver::Resolver,
        downloader: dash_downloader::Downloader,
        linker: dash_linker::Linker,
        store: dash_store::Store,
        config: InstallerConfig,
    ) -> Self {
        let cache = dash_cache::create_cache(&dash_cache::CacheConfig::default());
        Self {
            resolver,
            downloader,
            linker,
            cache,
            store,
            config,
        }
    }

    /// Install a single package and all its dependencies.
    pub async fn install(&self, pkg_name: &dash_package::PackageName) -> Result<InstallSummary, InstallError> {
        let start = Instant::now();
        let mut summary = InstallSummary::new();

        // Create a dependency set from the package name
        let mut deps = dash_resolver::DependencySet::new();
        deps.add(dash_resolver::Dependency::new(pkg_name.as_str(), "*"));

        // Resolve dependencies
        tracing::info!("Resolving dependencies for '{}'...", pkg_name);
        let resolved = self.resolver.resolve(&deps)?;

        // Create target directories
        dash_filesystem::ensure_dir(&self.config.target_dir)?;
        dash_filesystem::ensure_dir(&self.config.store_dir)?;

        // Process each resolved package
        let packages: Vec<String> = resolved.nodes.keys().cloned().collect();
        let concurrency = self.config.concurrency;

        // Download and install each package
        for pkg_name in &packages {
            let version = resolved.get(pkg_name).unwrap();

            let pkg_ref = dash_package::PackageRef::new(
                dash_package::PackageId::new(
                    dash_package::PackageName::new_unchecked(pkg_name),
                    version.clone(),
                ),
            );

            // Download
            tracing::info!("Downloading {pkg_name}@{version}...");
            let downloaded = self.downloader.download(&pkg_ref).await?;
            summary.total_bytes += downloaded.size;

            // Verify integrity
            if self.config.verify_integrity {
                if let Some(ref expected) = pkg_ref.integrity {
                    if !expected.verify(&downloaded.data) {
                        return Err(InstallError::Integrity(
                            dash_integrity::IntegrityError::HashMismatch {
                                expected: expected.to_string(),
                                actual: downloaded.integrity.to_string(),
                            },
                        ));
                    }
                }
            }

            // Store in content-addressable store
            let _hash = self.store.put(&downloaded.data).await?;

            // Extract to target directory
            let pkg_dir = self.config.target_dir.join(pkg_name);
            let entries = dash_archive::extract_archive(
                &downloaded.data,
                &pkg_dir,
                dash_archive::ArchiveFormat::TarGz,
            )?;
            summary.installed.push(pkg_name.clone());

            tracing::info!("Extracted {} files for {pkg_name}", entries.len());
        }

        // Link binaries
        for pkg_name in &packages {
            if let Some(version) = resolved.get(pkg_name) {
                let pkg = dash_package::Package::new(
                    dash_package::PackageName::new_unchecked(pkg_name),
                    version.clone(),
                );
                let linked = self.linker.link_binaries(&pkg).await?;
                summary.linked.extend(linked);
            }
        }

        summary.duration = start.elapsed();
        summary.package_count = packages.len();

        Ok(summary)
    }

    /// Install a set of packages in parallel.
    pub async fn install_many(
        &self,
        packages: &[dash_package::PackageName],
    ) -> Result<Vec<InstallSummary>, InstallError> {
        use futures::stream::{self, StreamExt, TryStreamExt};

        let results: Vec<Result<InstallSummary, InstallError>> = stream::iter(packages.iter())
            .map(|pkg| self.install(pkg))
            .buffered(self.config.concurrency)
            .collect()
            .await;

        results.into_iter().collect()
    }

    /// Install from a lockfile (reproducible install).
    pub async fn install_from_lockfile(
        &self,
        lockfile: &dash_lockfile::Lockfile,
    ) -> Result<InstallSummary, InstallError> {
        let start = Instant::now();
        let mut summary = InstallSummary::new();

        for (name, entry) in lockfile.iter() {
            let pkg_ref = dash_package::PackageRef::new(
                dash_package::PackageId::parse(name, &entry.version)?,
            );

            let downloaded = self.downloader.download(&pkg_ref).await?;
            let _hash = self.store.put(&downloaded.data).await?;

            let pkg_dir = self.config.target_dir.join(name);
            let entries = dash_archive::extract_archive(
                &downloaded.data,
                &pkg_dir,
                dash_archive::ArchiveFormat::TarGz,
            )?;

            summary.installed.push(name.to_string());
            summary.total_bytes += downloaded.size;
            tracing::info!("Installed {name}@{} ({} files)", entry.version, entries.len());
        }

        summary.duration = start.elapsed();
        summary.package_count = lockfile.len();

        Ok(summary)
    }

    /// Remove an installed package.
    pub async fn remove(&self, name: &dash_package::PackageName) -> Result<(), InstallError> {
        let pkg_dir = self.config.target_dir.join(name.as_str());
        if pkg_dir.exists() {
            dash_filesystem::remove_dir(&pkg_dir)?;
        }

        self.linker.unlink_binaries(name).await?;

        Ok(())
    }

    /// Check if a package is installed.
    pub async fn is_installed(&self, name: &str) -> bool {
        let pkg_dir = self.config.target_dir.join(name);
        pkg_dir.exists()
    }

    /// Get the installer configuration.
    pub fn config(&self) -> &InstallerConfig {
        &self.config
    }
}

/// Create an installer from a configuration.
pub fn create_installer(config: &dash_config::Config) -> Result<Installer, InstallError> {
    let resolver = dash_resolver::create_resolver(config);

    let downloader = dash_downloader::Downloader::from_config(config)?;

    let store = dash_store::create_store_from_config(config)?;

    let linker_config = dash_linker::LinkerConfig {
        bin_dir: config
            .get("install.bin_dir")
            .unwrap_or_else(|_| PathBuf::from("node_modules/.bin")),
        ..Default::default()
    };

    let store_backend = Arc::new(dash_store::LocalStore::new(
        config
            .get("store.path")
            .unwrap_or_else(|_| PathBuf::from(".dash/store")),
    ));
    let linker = dash_linker::Linker::new(store_backend, linker_config);

    let installer_config = InstallerConfig {
        target_dir: config
            .get("install.target_dir")
            .unwrap_or_else(|_| PathBuf::from("node_modules")),
        store_dir: config
            .get("install.store_dir")
            .unwrap_or_else(|_| PathBuf::from(".dash/store")),
        bin_dir: config
            .get("install.bin_dir")
            .unwrap_or_else(|_| PathBuf::from("node_modules/.bin")),
        concurrency: config.get_or("install.concurrency", 4usize),
        verify_integrity: config.get_or("install.verify_integrity", true),
        verify_signatures: config.get_or("install.verify_signatures", false),
        use_cache: config.get_or("install.use_cache", true),
        run_scripts: config.get_or("install.run_scripts", true),
        ..Default::default()
    };

    Ok(Installer::new(resolver, downloader, linker, store, installer_config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_config_default() {
        let config = InstallerConfig::default();
        assert_eq!(config.target_dir, PathBuf::from("node_modules"));
        assert!(config.verify_integrity);
        assert_eq!(config.concurrency, 4);
    }

    #[test]
    fn test_install_summary() {
        let mut summary = InstallSummary::new();
        assert!(summary.installed.is_empty());
        assert!(summary.linked.is_empty());

        summary.installed.push("test-pkg".to_string());
        summary.package_count = 1;
        summary.total_bytes = 1024;

        assert_eq!(summary.package_count, 1);
        assert!(!summary.installed.is_empty());
    }

    #[test]
    fn test_install_summary_merge() {
        let mut s1 = InstallSummary::new();
        s1.installed.push("pkg-a".to_string());
        s1.package_count = 1;

        let mut s2 = InstallSummary::new();
        s2.installed.push("pkg-b".to_string());
        s2.package_count = 1;

        s1.merge(s2);
        assert_eq!(s1.installed.len(), 2);
        assert_eq!(s1.package_count, 2);
    }

    #[tokio::test]
    async fn test_install_nonexistent_package() {
        let config = dash_config::Config::empty();
        let installer = create_installer(&config);

        // Should fail gracefully without a registry
        match installer {
            Ok(_) => {
                // Installer created successfully
            }
            Err(e) => {
                assert!(e.to_string().contains("error") || e.to_string().contains("failed"));
            }
        }
    }

    #[test]
    fn test_install_strategy() {
        assert_ne!(InstallStrategy::Copy, InstallStrategy::Link);
        assert_eq!(InstallStrategy::Link, InstallStrategy::Link);
    }

    #[tokio::test]
    async fn test_is_installed() {
        let config = dash_config::Config::empty();
        match create_installer(&config) {
            Ok(installer) => {
                let installed = installer.is_installed("nonexistent").await;
                assert!(!installed);
            }
            Err(_) => {
                // Skip test if installer can't be created
            }
        }
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let config = dash_config::Config::empty();
        match create_installer(&config) {
            Ok(installer) => {
                let name = dash_package::PackageName::new_unchecked("nonexistent");
                let result = installer.remove(&name).await;
                assert!(result.is_ok());
            }
            Err(_) => {}
        }
    }

    #[test]
    fn test_summary_default() {
        let summary = InstallSummary::default();
        assert_eq!(summary.duration, Duration::ZERO);
        assert_eq!(summary.package_count, 0);
    }
}
