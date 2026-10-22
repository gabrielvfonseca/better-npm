//! Integration test utilities: test fixtures, mock registries, helpers.
//!
//! Provides building blocks for writing comprehensive tests for
//! Bnpm packages and subsystems.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Error type for test utilities.
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] bnpm_config::ConfigError),
    #[error("assertion error: {0}")]
    Assertion(String),
    #[error("timeout: {0}")]
    Timeout(String),
}

/// Create a temporary test configuration with sensible defaults.
pub fn create_test_config() -> bnpm_config::Config {
    let mut config = bnpm_config::Config::empty();
    config.set("registry.url", "https://test-registry.bnpm.sh");
    config.set("store.backend", "local");
    config.set("store.path", "./test-store");
    config.set("cache.backend", "memory");
    config.set("cache.max_size", 1000);
    config.set("runtime.worker_threads", 2);
    config.set("log.level", "debug");
    config.set("log.format", "text");
    config
}

/// Create a temporary directory that is cleaned up on drop.
pub fn create_temp_dir() -> Result<bnpm_filesystem::TempDir, TestError> {
    Ok(bnpm_filesystem::TempDir::new().map_err(|e| TestError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?)
}

/// Create a test package with the given name and version.
pub fn create_test_package(name: &str, version: &str) -> bnpm_package::Package {
    let pkg_name = bnpm_package::PackageName::new(name).unwrap_or_else(|_| {
        bnpm_package::PackageName::new_unchecked(name)
    });
    let pkg_version: semver::Version = version.parse().expect("valid semver");
    let mut pkg = bnpm_package::Package::new(pkg_name, pkg_version);
    pkg.description = Some(format!("Test package {name}"));
    pkg.author = Some("Bnpm Test Suite".to_string());
    pkg.license = Some("MIT".to_string());
    pkg.add_file("index.js", b"module.exports = {};" .to_vec());
    pkg.add_file("package.json", format!("{{\"name\": \"{name}\", \"version\": \"{version}\"}}").into_bytes());
    pkg
}

/// Run a closure with tracing initialized.
pub fn with_tracing(f: impl FnOnce()) {
    let config = bnpm_logger::LogConfig {
        level: "error".to_string(),
        format: bnpm_logger::LogFormat::Text,
        output: bnpm_logger::LogOutput::Stdout,
        include_spans: false,
    };
    let _ = bnpm_logger::init(&config);
    f();
}

/// Run an async test, blocking the current thread until completion.
pub fn async_test<F: Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
    rt.block_on(f)
}

/// Generate a cryptographically random alphanumeric string.
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a unique string using a counter.
pub fn unique_name(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{count}")
}

/// Wait for a condition to become true, with a timeout.
pub async fn wait_for<F, Fut>(poll: F, timeout: std::time::Duration) -> Result<(), TestError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if poll().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    Err(TestError::Timeout(format!(
        "condition not met within {:?}",
        timeout
    )))
}

/// Assert that an async operation completes within a timeout.
pub async fn assert_timeout<F, Fut, T>(f: F, timeout: std::time::Duration) -> Result<T, TestError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    tokio::time::timeout(timeout, f())
        .await
        .map_err(|_| TestError::Timeout(format!("operation timed out after {:?}", timeout)))
}

/// Create a mock package store for testing.
pub fn create_test_store() -> bnpm_store::Store {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let backend = std::sync::Arc::new(bnpm_store::LocalStore::new(dir.path().to_path_buf()));
    bnpm_store::Store::new(backend)
}

/// Create a test cache backend.
pub fn create_test_cache() -> std::sync::Arc<dyn bnpm_cache::CacheBackend> {
    let config = bnpm_cache::CacheConfig {
        max_size: 1000,
        ..Default::default()
    };
    bnpm_cache::create_cache(&config)
}

/// Assert that two values are equal with a custom message.
pub fn assert_eq_msg<T: std::fmt::Debug + PartialEq>(left: T, right: T, msg: &str) {
    assert_eq!(left, right, "{msg}");
}

/// Read a fixture file from the test fixtures directory.
pub fn read_fixture(path: &str) -> Result<Vec<u8>, TestError> {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path);
    Ok(std::fs::read(&fixture_path)?)
}

/// A mock registry for testing publish/fetch flows.
pub struct MockRegistry {
    packages: std::sync::Mutex<std::collections::HashMap<String, bnpm_registry_client::RegistryPackage>>,
}

impl MockRegistry {
    /// Create a new empty mock registry.
    pub fn new() -> Self {
        Self {
            packages: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Add a package to the mock registry.
    pub fn add_package(&self, pkg: bnpm_registry_client::RegistryPackage) {
        let mut packages = self.packages.lock().unwrap();
        packages.insert(pkg.name.clone(), pkg);
    }

    /// Get a package from the mock registry.
    pub fn get_package(&self, name: &str) -> Option<bnpm_registry_client::RegistryPackage> {
        let packages = self.packages.lock().unwrap();
        packages.get(name).cloned()
    }

    /// Check if a package exists.
    pub fn has_package(&self, name: &str) -> bool {
        let packages = self.packages.lock().unwrap();
        packages.contains_key(name)
    }

    /// Return the number of packages.
    pub fn package_count(&self) -> usize {
        let packages = self.packages.lock().unwrap();
        packages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert_eq!(
            config.get::<String>("registry.url").unwrap(),
            "https://test-registry.bnpm.sh"
        );
    }

    #[test]
    fn test_create_temp_dir() {
        let dir = create_temp_dir().unwrap();
        let path = dir.path().to_path_buf();
        assert!(path.exists());
        drop(dir);
        assert!(!path.exists());
    }

    #[test]
    fn test_create_test_package() {
        let pkg = create_test_package("test-pkg", "1.0.0");
        assert_eq!(pkg.name.as_str(), "test-pkg");
        assert_eq!(pkg.version.to_string(), "1.0.0");
        assert_eq!(pkg.files.len(), 2);
        assert_eq!(pkg.description.as_deref(), Some("Test package test-pkg"));
    }

    #[test]
    fn test_random_string() {
        let s1 = random_string(10);
        let s2 = random_string(10);
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_unique_name() {
        let n1 = unique_name("test");
        let n2 = unique_name("test");
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_with_tracing() {
        with_tracing(|| {
            assert!(true);
        });
    }

    #[test]
    fn test_async_test() {
        let result = async_test(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_mock_registry() {
        let registry = MockRegistry::new();
        assert_eq!(registry.package_count(), 0);

        let pkg = bnpm_registry_client::RegistryPackage {
            name: "mock-pkg".to_string(),
            scope: None,
            version: "1.0.0".to_string(),
            description: Some("A mock package".to_string()),
            versions: std::collections::BTreeMap::new(),
            dist_tags: std::collections::BTreeMap::new(),
            author: None,
            license: None,
            homepage: None,
            repository: None,
            time: None,
        };
        registry.add_package(pkg);
        assert!(registry.has_package("mock-pkg"));
        assert_eq!(registry.package_count(), 1);
    }

    #[test]
    fn test_create_test_store() {
        let store = create_test_store();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let key = store.put(b"test data").await.unwrap();
            assert!(store.exists(&key).await.unwrap());
        });
    }

    #[test]
    fn test_create_test_cache() {
        let cache = create_test_cache();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            bnpm_cache::cache_set(&*cache, "key", &"value".to_string(), None).await.unwrap();
            assert!(cache.exists("key").await.unwrap());
        });
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(
            || async { false },
            std::time::Duration::from_millis(100),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wait_for_success() {
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let r = ready.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            r.store(true, Ordering::SeqCst);
        });

        let result = wait_for(
            move || {
                let ready = ready.clone();
                async move { ready.load(Ordering::SeqCst) }
            },
            std::time::Duration::from_secs(5),
        )
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_eq_msg() {
        assert_eq_msg(1, 1, "should be equal");
    }
}
