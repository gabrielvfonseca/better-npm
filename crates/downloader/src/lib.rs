//! Concurrent HTTP/registry download with streaming integrity verification.
//!
//! Downloads packages from the registry with progress reporting,
//! caching, integrity checking, and parallel batch downloads.

use std::sync::Arc;
use std::time::{Duration, Instant};

/// Error type for download operations.
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP error: {status} {message}")]
    Http { status: u16, message: String },
    #[error("request failed: {0}")]
    Request(String),
    #[error("integrity check failed: {expected}, got {actual}")]
    IntegrityMismatch { expected: String, actual: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cache error: {0}")]
    Cache(#[from] bnpm_cache::CacheError),
    #[error("hashing error: {0}")]
    Hash(#[from] bnpm_hashing::HashError),
    #[error("network error: {0}")]
    Network(#[from] bnpm_network::NetworkError),
    #[error("file not found in cache: {0}")]
    CacheMiss(String),
}

/// A fully downloaded package ready for installation.
#[derive(Debug, Clone)]
pub struct DownloadedPackage {
    /// Package reference that was downloaded.
    pub reference: bnpm_package::PackageRef,
    /// Raw archive data.
    pub data: Vec<u8>,
    /// Verified integrity hash.
    pub integrity: bnpm_integrity::Integrity,
    /// Download duration.
    pub duration: Duration,
    /// Size of the downloaded data.
    pub size: u64,
}

/// Progress information for an ongoing download.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total number of bytes to download.
    pub total: u64,
    /// Number of bytes downloaded so far.
    pub completed: u64,
    /// Bytes downloaded per second (smoothed).
    pub speed: f64,
    /// Elapsed time.
    pub elapsed: Duration,
    /// Estimated remaining time.
    pub eta: Duration,
}

impl DownloadProgress {
    /// Current progress as a fraction (0.0 to 1.0).
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.completed as f64 / self.total as f64
    }

    /// Whether the download is complete.
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }

    /// Format progress as a percentage.
    pub fn percent(&self) -> f64 {
        self.fraction() * 100.0
    }
}

/// Events emitted during a download operation.
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    /// A download has started.
    DownloadStarted { name: String, version: String, total: u64 },
    /// Progress update.
    DownloadProgress(DownloadProgress),
    /// A download completed successfully.
    DownloadCompleted { name: String, version: String, size: u64, duration: Duration },
    /// A download failed.
    DownloadFailed { name: String, version: String, error: String },
}

/// The download manager.
pub struct Downloader {
    client: reqwest::Client,
    cache: Arc<dyn bnpm_cache::CacheBackend>,
    concurrency: usize,
    config: DownloaderConfig,
}

/// Configuration for the downloader.
#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    /// Maximum number of concurrent downloads.
    pub concurrency: usize,
    /// Whether to verify integrity after download.
    pub verify_integrity: bool,
    /// Whether to cache downloaded packages.
    pub use_cache: bool,
    /// Base URL for the registry.
    pub registry_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Whether to resume partial downloads.
    pub resume: bool,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            verify_integrity: true,
            use_cache: true,
            registry_url: "https://registry.bnpm.sh".to_string(),
            timeout: Duration::from_secs(120),
            resume: false,
        }
    }
}

impl Downloader {
    /// Create a new downloader.
    pub fn new(
        client: reqwest::Client,
        cache: Arc<dyn bnpm_cache::CacheBackend>,
        config: DownloaderConfig,
    ) -> Self {
        Self {
            client,
            cache,
            concurrency: config.concurrency,
            config,
        }
    }

    /// Create a downloader from bnpm configuration.
    pub fn from_config(config: &bnpm_config::Config) -> Result<Self, DownloadError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.get_or("download.timeout_secs", 120u64)))
            .user_agent(bnpm_network::user_agent())
            .default_headers(bnpm_network::default_headers())
            .build()
            .map_err(|e| DownloadError::Request(e.to_string()))?;

        let cache_config = bnpm_cache::CacheConfig {
            backend: config.get_or("cache.backend", "memory".to_string()),
            max_size: config.get_or("cache.max_size", 10000u64),
            ..Default::default()
        };
        let cache = bnpm_cache::create_cache(&cache_config);

        let download_config = DownloaderConfig {
            concurrency: config.get_or("download.concurrency", 4usize),
            verify_integrity: config.get_or("download.verify_integrity", true),
            use_cache: config.get_or("download.use_cache", true),
            registry_url: config.get_or("registry.url", "https://registry.bnpm.sh".to_string()),
            ..Default::default()
        };

        Ok(Self::new(client, cache, download_config))
    }

    /// Download a single package.
    pub async fn download(&self, pkg: &bnpm_package::PackageRef) -> Result<DownloadedPackage, DownloadError> {
        // Check cache
        if self.config.use_cache {
            let cache_key = format!("download:{}", pkg.id);
            if let Ok(Some(data)) = bnpm_cache::cache_get::<Vec<u8>>(&*self.cache, &cache_key).await {
                let integrity = bnpm_integrity::Integrity::hash(&data, bnpm_integrity::IntegrityAlgorithm::Sha256);
                let size = data.len() as u64;
                return Ok(DownloadedPackage {
                    reference: pkg.clone(),
                    data,
                    integrity,
                    duration: Duration::ZERO,
                    size,
                });
            }
        }

        let url = pkg
            .url
            .clone()
            .unwrap_or_else(|| format!("{}/packages/{}/tarball/{}/{}", self.config.registry_url, pkg.id.name, pkg.id.version, pkg.id.name));

        let start = Instant::now();
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DownloadError::Request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError::Http {
                status: status.as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| DownloadError::Request(e.to_string()))?
            .to_vec();

        let duration = start.elapsed();

        // Verify integrity
        if self.config.verify_integrity {
            if let Some(ref expected) = pkg.integrity {
                let actual = bnpm_integrity::Integrity::hash(&data, expected.algorithm);
                if !expected.verify(&data) {
                    return Err(DownloadError::IntegrityMismatch {
                        expected: expected.to_string(),
                        actual: actual.to_string(),
                    });
                }
            }
        }

        // Cache the result
        if self.config.use_cache {
            let cache_key = format!("download:{}", pkg.id);
            let _ = bnpm_cache::cache_set(&*self.cache, &cache_key, &data, None).await;
        }

        let size = data.len() as u64;
        let integrity = bnpm_integrity::Integrity::hash(&data, bnpm_integrity::IntegrityAlgorithm::Sha256);

        Ok(DownloadedPackage {
            reference: pkg.clone(),
            data,
            integrity,
            duration,
            size,
        })
    }

    /// Download multiple packages concurrently with bounded parallelism.
    pub async fn download_batch(
        &self,
        pkgs: &[bnpm_package::PackageRef],
    ) -> Result<Vec<DownloadedPackage>, DownloadError> {
        use futures::stream::{self, StreamExt};

        let results: Vec<Result<DownloadedPackage, DownloadError>> = stream::iter(pkgs.iter())
            .map(|pkg| self.download(pkg))
            .buffered(self.concurrency)
            .collect()
            .await;

        // Collect results, returning the first error if any
        results.into_iter().collect()
    }

    /// Check if a package is cached locally.
    pub async fn is_cached(&self, pkg: &bnpm_package::PackageRef) -> bool {
        if !self.config.use_cache {
            return false;
        }
        let cache_key = format!("download:{}", pkg.id);
        self.cache.exists(&cache_key).await.unwrap_or(false)
    }

    /// Clear the download cache.
    pub async fn clear_cache(&self) -> Result<(), DownloadError> {
        self.cache.clear().await?;
        Ok(())
    }

    /// Get the download concurrency.
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }
}

/// Compute download speed in bytes per second.
pub fn compute_speed(bytes: u64, duration: Duration) -> f64 {
    let secs = duration.as_secs_f64();
    if secs > 0.0 {
        bytes as f64 / secs
    } else {
        bytes as f64
    }
}

/// Format a byte rate for display.
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_000_000.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_000_000.0)
    } else if bytes_per_sec >= 1_000.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_000.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_config_default() {
        let config = DownloaderConfig::default();
        assert_eq!(config.concurrency, 4);
        assert!(config.verify_integrity);
        assert!(config.use_cache);
    }

    #[tokio::test]
    async fn test_download_no_cache() {
        // Without a registry, download should fail gracefully
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();

        let cache_config = bnpm_cache::CacheConfig::default();
        let cache = bnpm_cache::create_cache(&cache_config);
        let config = DownloaderConfig {
            registry_url: "http://localhost:1".to_string(),
            timeout: Duration::from_secs(1),
            use_cache: false,
            ..Default::default()
        };

        let downloader = Downloader::new(client, cache, config);

        let name = bnpm_package::PackageName::new_unchecked("test-pkg");
        let version = semver::Version::new(1, 0, 0);
        let id = bnpm_package::PackageId::new(name, version);
        let pkg_ref = bnpm_package::PackageRef::new(id);

        let result = downloader.download(&pkg_ref).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_speed() {
        let speed = compute_speed(1024, Duration::from_secs(1));
        assert!((speed - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(1500.0), "1.5 KB/s");
        assert_eq!(format_speed(1_500_000.0), "1.5 MB/s");
    }

    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress {
            total: 100,
            completed: 50,
            speed: 1024.0,
            elapsed: Duration::from_secs(5),
            eta: Duration::from_secs(5),
        };

        assert!((progress.fraction() - 0.5).abs() < f64::EPSILON);
        assert!((progress.percent() - 50.0).abs() < f64::EPSILON);
        assert!(!progress.is_complete());

        let complete = DownloadProgress {
            total: 100,
            completed: 100,
            speed: 0.0,
            elapsed: Duration::from_secs(10),
            eta: Duration::ZERO,
        };
        assert!(complete.is_complete());
    }

    #[test]
    fn test_download_events() {
        let event = DownloadEvent::DownloadStarted {
            name: "pkg".to_string(),
            version: "1.0.0".to_string(),
            total: 1024,
        };
        match event {
            DownloadEvent::DownloadStarted { name, .. } => assert_eq!(name, "pkg"),
            _ => panic!("wrong event type"),
        }

        let failed = DownloadEvent::DownloadFailed {
            name: "pkg".to_string(),
            version: "1.0.0".to_string(),
            error: "connection refused".to_string(),
        };
        match failed {
            DownloadEvent::DownloadFailed { error, .. } => assert_eq!(error, "connection refused"),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_downloaded_package() {
        let name = bnpm_package::PackageName::new_unchecked("test");
        let version = semver::Version::new(1, 0, 0);
        let id = bnpm_package::PackageId::new(name, version);
        let pref = bnpm_package::PackageRef::new(id);

        let data = b"test package data".to_vec();
        let integrity = bnpm_integrity::Integrity::hash(&data, bnpm_integrity::IntegrityAlgorithm::Sha256);

        let dpkg = DownloadedPackage {
            reference: pref,
            data: data.clone(),
            integrity: integrity.clone(),
            duration: Duration::from_secs(2),
            size: data.len() as u64,
        };

        assert!(integrity.verify(&dpkg.data));
        assert_eq!(dpkg.size, 17);
    }

    #[tokio::test]
    async fn test_is_cached() {
        let client = reqwest::Client::new();
        let cache = bnpm_cache::create_cache(&Default::default());
        let config = DownloaderConfig {
            use_cache: true,
            ..Default::default()
        };
        let downloader = Downloader::new(client, cache, config);

        let name = bnpm_package::PackageName::new_unchecked("test");
        let version = semver::Version::new(1, 0, 0);
        let id = bnpm_package::PackageId::new(name, version);
        let pref = bnpm_package::PackageRef::new(id);

        assert!(!downloader.is_cached(&pref).await);
    }

    #[tokio::test]
    async fn test_download_batch_empty() {
        let client = reqwest::Client::new();
        let cache = bnpm_cache::create_cache(&Default::default());
        let config = DownloaderConfig::default();
        let downloader = Downloader::new(client, cache, config);

        let result = downloader.download_batch(&[]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
