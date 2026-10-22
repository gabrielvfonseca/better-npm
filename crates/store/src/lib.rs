//! Persistent and object storage abstraction with local and cloud backends.
//!
//! Provides a content-addressable store where data is keyed by its
//! hash, supporting local filesystem and object store (S3, GCS, Azure) backends.

use std::path::PathBuf;
use std::sync::Arc;

/// Error type for store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cache error: {0}")]
    Cache(#[from] bnpm_cache::CacheError),
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error("hash mismatch: expected {expected}, computed {computed}")]
    HashMismatch { expected: String, computed: String },
    #[error("backend error: {0}")]
    Backend(String),
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("object store error: {0}")]
    ObjectStore(#[from] object_store::Error),
}

/// A generic backend for content-addressable storage.
#[async_trait::async_trait]
pub trait StoreBackend: Send + Sync {
    /// Retrieve data by its content hash.
    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError>;
    /// Store data, returning its content hash.
    async fn put(&self, data: &[u8]) -> Result<String, StoreError>;
    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, StoreError>;
    /// Remove data by key.
    async fn remove(&self, key: &str) -> Result<(), StoreError>;
    /// List all keys in the store.
    async fn list(&self) -> Result<Vec<String>, StoreError>;
}

/// Filesystem-backed content-addressable store.
///
/// Stores blobs at `{base_path}/{key_prefix}/{key}`.
pub struct LocalStore {
    base_path: PathBuf,
    cache: Arc<dyn bnpm_cache::CacheBackend>,
}

impl LocalStore {
    /// Create a new local store rooted at `base_path`.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            cache: bnpm_cache::create_cache(&bnpm_cache::CacheConfig::default()),
        }
    }

    /// Create a new local store with a cache backend.
    pub fn new_with_cache(base_path: PathBuf, cache: Arc<dyn bnpm_cache::CacheBackend>) -> Self {
        Self { base_path, cache }
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        let (prefix, rest) = if key.len() > 4 {
            (&key[..2], &key[2..])
        } else {
            ("_", key)
        };
        self.base_path.join(prefix).join(rest)
    }
}

#[async_trait::async_trait]
impl StoreBackend for LocalStore {
    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        // Check cache first
        if let Some(cached) = bnpm_cache::cache_get::<Vec<u8>>(&*self.cache, &format!("store:{}", key)).await? {
            return Ok(cached);
        }

        let path = self.key_to_path(key);
        let data = tokio::fs::read(&path).await?;

        // Cache for future lookups
        let _ = bnpm_cache::cache_set(&*self.cache, &format!("store:{}", key), &data, None).await;

        Ok(data)
    }

    async fn put(&self, data: &[u8]) -> Result<String, StoreError> {
        let hash = bnpm_hashing::hash_sha256(data);
        let key = hex::encode(hash);
        let path = self.key_to_path(&key);

        // Create parent directory
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, data).await?;

        // Update cache
        let _ = bnpm_cache::cache_set(&*self.cache, &format!("store:{}", key), &data.to_vec(), None).await;

        Ok(key)
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        // Check cache
        if self.cache.exists(&format!("store:{}", key)).await? {
            return Ok(true);
        }

        let path = self.key_to_path(key);
        Ok(path.exists())
    }

    async fn remove(&self, key: &str) -> Result<(), StoreError> {
        let path = self.key_to_path(key);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        self.cache.remove(&format!("store:{}", key)).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>, StoreError> {
        let mut keys = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let prefix = entry.file_name().to_string_lossy().to_string();
                let mut sub_dir = tokio::fs::read_dir(entry.path()).await?;
                while let Some(sub_entry) = sub_dir.next_entry().await? {
                    if sub_entry.file_type().await?.is_file() {
                        let name = sub_entry.file_name().to_string_lossy().to_string();
                        keys.push(format!("{}{}", prefix, name));
                    }
                }
            }
        }

        Ok(keys)
    }
}

/// Object store backend (S3, GCS, Azure) using the `object_store` crate.
pub struct ObjectStoreBackend {
    inner: Arc<dyn object_store::ObjectStore>,
    prefix: String,
    cache: Arc<dyn bnpm_cache::CacheBackend>,
}

impl ObjectStoreBackend {
    /// Create a new object store backend.
    pub fn new(
        inner: Arc<dyn object_store::ObjectStore>,
        prefix: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            prefix: prefix.into(),
            cache: bnpm_cache::create_cache(&bnpm_cache::CacheConfig::default()),
        }
    }

    /// Create an S3-backed store.
    pub async fn s3(bucket: &str, region: &str, prefix: impl Into<String>) -> Result<Self, StoreError> {
        use object_store::aws::AmazonS3Builder;
        let store = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_region(region)
            .build()?;
        Ok(Self::new(Arc::new(store), prefix))
    }

    /// Create a GCS-backed store.
    pub async fn gcs(bucket: &str, prefix: impl Into<String>) -> Result<Self, StoreError> {
        use object_store::gcp::GoogleCloudStorageBuilder;
        let store = GoogleCloudStorageBuilder::new()
            .with_bucket_name(bucket)
            .build()?;
        Ok(Self::new(Arc::new(store), prefix))
    }

    fn object_path(&self, key: &str) -> object_store::path::Path {
        let path = format!("{}/{}", self.prefix, key);
        object_store::path::Path::from(path)
    }
}

#[async_trait::async_trait]
impl StoreBackend for ObjectStoreBackend {
    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        // Check cache
        if let Some(cached) = bnpm_cache::cache_get::<Vec<u8>>(&*self.cache, &format!("ostore:{}", key)).await? {
            return Ok(cached);
        }

        let path = self.object_path(key);
        use object_store::ObjectStore;
        let result = self.inner.get(&path).await?;
        let data = result.bytes().await?.to_vec();

        // Cache
        let _ = bnpm_cache::cache_set(&*self.cache, &format!("ostore:{}", key), &data, None).await;

        Ok(data)
    }

    async fn put(&self, data: &[u8]) -> Result<String, StoreError> {
        let hash = bnpm_hashing::hash_sha256(data);
        let key = hex::encode(hash);
        let path = self.object_path(&key);

        use object_store::ObjectStore;
        self.inner.put(&path, data.to_vec().into()).await?;

        // Cache
        let _ = bnpm_cache::cache_set(&*self.cache, &format!("ostore:{}", key), &data.to_vec(), None).await;

        Ok(key)
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        if self.cache.exists(&format!("ostore:{}", key)).await? {
            return Ok(true);
        }
        let path = self.object_path(key);
        use object_store::ObjectStore;
        Ok(self.inner.head(&path).await.is_ok())
    }

    async fn remove(&self, key: &str) -> Result<(), StoreError> {
        let path = self.object_path(key);
        use object_store::ObjectStore;
        self.inner.delete(&path).await?;
        self.cache.remove(&format!("ostore:{}", key)).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>, StoreError> {
        let mut keys = Vec::new();
        use futures::StreamExt;
        use object_store::ObjectStore;
        let prefix = object_store::path::Path::from(self.prefix.clone());
        let mut result = self.inner.list(Some(&prefix));

        while let Some(meta) = result.next().await {
            let meta = meta?;
            let key = meta.location.to_string();
            // Strip prefix
            if let Some(stripped) = key.strip_prefix(&format!("{}/", self.prefix)) {
                keys.push(stripped.to_string());
            }
        }

        Ok(keys)
    }
}

use futures::StreamExt;

/// A content-addressable store with pluggable backend.
pub struct Store {
    backend: Arc<dyn StoreBackend>,
}

impl Store {
    /// Create a new store with the given backend.
    pub fn new(backend: Arc<dyn StoreBackend>) -> Self {
        Self { backend }
    }

    /// Retrieve data by content hash.
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        self.backend.get(key).await
    }

    /// Store data, returning the content hash (SHA-256).
    pub async fn put(&self, data: &[u8]) -> Result<String, StoreError> {
        self.backend.put(data).await
    }

    /// Check if a key exists in the store.
    pub async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        self.backend.exists(key).await
    }

    /// Remove data by key.
    pub async fn remove(&self, key: &str) -> Result<(), StoreError> {
        self.backend.remove(key).await
    }

    /// List all stored keys.
    pub async fn list(&self) -> Result<Vec<String>, StoreError> {
        self.backend.list().await
    }

    /// Verify that stored data matches its key (hash).
    pub async fn verify(&self, key: &str) -> Result<bool, StoreError> {
        let data = self.backend.get(key).await?;
        let computed = bnpm_hashing::hash_sha256(&data);
        let computed_key = hex::encode(computed);
        Ok(computed_key == key)
    }
}

/// Create a local filesystem-based store at the given path.
pub fn create_local_store(base_path: PathBuf) -> Store {
    Store::new(Arc::new(LocalStore::new(base_path)))
}

/// Create a store from a configuration.
pub fn create_store_from_config(config: &bnpm_config::Config) -> Result<Store, StoreError> {
    let backend: String = config.get_or("store.backend", "local".to_string());

    match backend.as_str() {
        "local" | "filesystem" => {
            let path: String = config.get_or("store.path", "./bnpm-store".to_string());
            Ok(Store::new(Arc::new(LocalStore::new(PathBuf::from(path)))))
        }
        "s3" => {
            let bucket: String = config.get_or("store.bucket", "bnpm-packages".to_string());
            let region: String = config.get_or("store.region", "us-east-1".to_string());
            let prefix: String = config.get_or("store.prefix", "packages".to_string());
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| StoreError::Backend(e.to_string()))?;
            rt.block_on(ObjectStoreBackend::s3(&bucket, &region, &prefix))
                .map(|backend| Store::new(Arc::new(backend)))
        }
        other => Err(StoreError::Backend(format!("unsupported store backend: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_local_store_put_get() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());
        let data = b"hello, content-addressable store!";

        let key = store.put(data).await.unwrap();
        assert_eq!(key.len(), 64); // SHA-256 hex

        let retrieved = store.get(&key).await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_local_store_exists() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());

        let key = store.put(b"test data").await.unwrap();
        assert!(store.exists(&key).await.unwrap());
        assert!(!store.exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_local_store_remove() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());

        let key = store.put(b"to be removed").await.unwrap();
        assert!(store.exists(&key).await.unwrap());
        store.remove(&key).await.unwrap();
        assert!(!store.exists(&key).await.unwrap());
    }

    #[tokio::test]
    async fn test_local_store_key_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());

        let result = store.get("nonexistent_key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_store_verify() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(Arc::new(LocalStore::new(dir.path().to_path_buf())));

        let key = store.put(b"verify me").await.unwrap();
        assert!(store.verify(&key).await.unwrap());
    }

    #[tokio::test]
    async fn test_store_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());

        store.put(b"data1").await.unwrap();
        store.put(b"data2").await.unwrap();

        let keys = store.list().await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_create_local_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = create_local_store(dir.path().to_path_buf());
        assert!(store.list().is_ok());
    }

    #[tokio::test]
    async fn test_put_deterministic_key() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        let store1 = LocalStore::new(dir1.path().to_path_buf());
        let store2 = LocalStore::new(dir2.path().to_path_buf());

        let data = b"deterministic content";
        let key1 = store1.put(data).await.unwrap();
        let key2 = store2.put(data).await.unwrap();
        assert_eq!(key1, key2);
    }
}
