//! Multi-tier caching layer with in-memory (Moka) and optional Redis backends.
//!
//! Provides a unified `CacheBackend` trait with interchangeable
//! implementations for local and distributed caching.

use std::sync::Arc;
use std::time::Duration;

/// Error type for cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache miss: {0}")]
    Miss(String),
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("backend error: {0}")]
    Backend(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Generic cache backend trait (dyn-safe).
///
/// Uses `Vec<u8>` and `serde_json::Value` to avoid generic type parameters
/// which would prevent `dyn` usage.
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a value from the cache by key. Returns serialized JSON bytes.
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Set a value in the cache from serialized bytes.
    async fn set_raw(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError>;

    /// Remove a value from the cache.
    async fn remove(&self, key: &str) -> Result<(), CacheError>;

    /// Check if a key exists in the cache.
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear all entries from the cache.
    async fn clear(&self) -> Result<(), CacheError>;
}

/// Convenience methods for typed serialization.
///
/// These are provided as free functions since the trait can only have dyn-safe methods.
pub async fn cache_get<T: serde::de::DeserializeOwned>(
    cache: &dyn CacheBackend,
    key: &str,
) -> Result<Option<T>, CacheError> {
    match cache.get_raw(key).await? {
        Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        None => Ok(None),
    }
}

pub async fn cache_set<T: serde::Serialize>(
    cache: &dyn CacheBackend,
    key: &str,
    value: &T,
    ttl: Option<Duration>,
) -> Result<(), CacheError> {
    let bytes = serde_json::to_vec(value)?;
    cache.set_raw(key, &bytes, ttl).await
}

/// In-memory cache backed by Moka.
pub struct MemoryCache {
    inner: moka::sync::Cache<String, bytes::Bytes>,
}

impl MemoryCache {
    /// Create a new memory cache with the given maximum size and default TTL.
    pub fn new(max_size: u64, default_ttl: Option<Duration>) -> Self {
        let mut builder = moka::sync::CacheBuilder::new(max_size);
        if let Some(ttl) = default_ttl {
            builder = builder.time_to_live(ttl);
        }
        Self {
            inner: builder.build(),
        }
    }
}

#[async_trait::async_trait]
impl CacheBackend for MemoryCache {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        match self.inner.get(key) {
            Some(bytes) => Ok(Some(bytes.to_vec())),
            None => Ok(None),
        }
    }

    async fn set_raw(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<(), CacheError> {
        self.inner.insert(key.to_string(), bytes::Bytes::copy_from_slice(value));
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        self.inner.invalidate(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Ok(self.inner.contains_key(key))
    }

    async fn clear(&self) -> Result<(), CacheError> {
        self.inner.invalidate_all();
        Ok(())
    }
}

/// Redis-backed cache.
pub struct RedisCache {
    client: redis::aio::ConnectionManager,
    default_ttl: Duration,
    namespace: String,
}

impl RedisCache {
    /// Create a new Redis cache from a Redis URL.
    pub async fn new(url: &str, default_ttl: Duration, namespace: String) -> Result<Self, CacheError> {
        let client = redis::Client::open(url)
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(Self {
            client: conn,
            default_ttl,
            namespace,
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }
}

#[async_trait::async_trait]
impl CacheBackend for RedisCache {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let prefixed = self.prefixed_key(key);
        let result: Option<Vec<u8>> = redis::Cmd::get(&prefixed)
            .query_async(&mut self.client.clone())
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(result)
    }

    async fn set_raw(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError> {
        let prefixed = self.prefixed_key(key);
        let ttl = ttl.unwrap_or(self.default_ttl);

        redis::Cmd::set_ex(&prefixed, value, ttl.as_secs())
            .query_async::<()>(&mut self.client.clone())
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let prefixed = self.prefixed_key(key);
        redis::Cmd::del(&prefixed)
            .query_async::<()>(&mut self.client.clone())
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let prefixed = self.prefixed_key(key);
        let result: bool = redis::Cmd::exists(&prefixed)
            .query_async(&mut self.client.clone())
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(result)
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let pattern = format!("{}:*", self.namespace);
        let mut conn = self.client.clone();
        let keys: Vec<String> = redis::Cmd::keys(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        if !keys.is_empty() {
            redis::Cmd::del(keys)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| CacheError::Backend(e.to_string()))?;
        }
        Ok(())
    }
}

/// Configuration for creating a cache backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheConfig {
    /// Which backend to use: "memory" or "redis".
    pub backend: String,
    /// Default TTL in seconds.
    pub ttl_secs: u64,
    /// Key namespace prefix (for Redis).
    pub namespace: String,
    /// Maximum number of entries (for memory cache).
    pub max_size: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            ttl_secs: 300,
            namespace: "bnpm".to_string(),
            max_size: 10_000,
        }
    }
}

/// Create a cache backend from a configuration.
pub fn create_cache(config: &CacheConfig) -> Arc<dyn CacheBackend> {
    match config.backend.as_str() {
        "redis" => {
            tracing::warn!("Redis cache not available in sync context; use create_cache_async");
            Arc::new(MemoryCache::new(
                config.max_size,
                Some(Duration::from_secs(config.ttl_secs)),
            ))
        }
        _ => Arc::new(MemoryCache::new(
            config.max_size,
            Some(Duration::from_secs(config.ttl_secs)),
        )),
    }
}

/// Create a cache backend asynchronously, supporting Redis.
pub async fn create_cache_async(config: &CacheConfig) -> Arc<dyn CacheBackend> {
    match config.backend.as_str() {
        "redis" => {
            let redis_url = std::env::var("BNPM_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            match RedisCache::new(
                &redis_url,
                Duration::from_secs(config.ttl_secs),
                config.namespace.clone(),
            )
            .await
            {
                Ok(cache) => Arc::new(cache),
                Err(e) => {
                    tracing::warn!("Failed to connect to Redis, falling back to memory cache: {e}");
                    Arc::new(MemoryCache::new(
                        config.max_size,
                        Some(Duration::from_secs(config.ttl_secs)),
                    ))
                }
            }
        }
        _ => Arc::new(MemoryCache::new(
            config.max_size,
            Some(Duration::from_secs(config.ttl_secs)),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_get_set() {
        let cache = MemoryCache::new(100, None);
        cache_set(&cache, "key1", &"value1".to_string(), None).await.unwrap();
        let result: Option<String> = cache_get(&cache, "key1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_memory_cache_miss() {
        let cache = MemoryCache::new(100, None);
        let result: Option<String> = cache_get(&cache, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_cache_exists() {
        let cache = MemoryCache::new(100, None);
        cache_set(&cache, "exists", &true, None).await.unwrap();
        assert!(cache.exists("exists").await.unwrap());
        assert!(!cache.exists("not_exists").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_cache_remove() {
        let cache = MemoryCache::new(100, None);
        cache_set(&cache, "toremove", &"value".to_string(), None).await.unwrap();
        cache.remove("toremove").await.unwrap();
        assert!(!cache.exists("toremove").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_cache_clear() {
        let cache = MemoryCache::new(100, None);
        cache_set(&cache, "a", &1, None).await.unwrap();
        cache_set(&cache, "b", &2, None).await.unwrap();
        cache.clear().await.unwrap();
        assert!(!cache.exists("a").await.unwrap());
        assert!(!cache.exists("b").await.unwrap());
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.backend, "memory");
        assert_eq!(config.ttl_secs, 300);
        assert_eq!(config.namespace, "bnpm");
        assert_eq!(config.max_size, 10_000);
    }
}
