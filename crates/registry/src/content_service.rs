use std::sync::Arc;

use bnpm_capsule::Capsule;
use bnpm_cid::Cid;
use bnpm_store::StoreError;
use bnpm_store::cas::CasStore;

#[derive(Clone)]
pub struct ContentService {
    store: Arc<CasStore>,
}

impl ContentService {
    pub fn new(store: CasStore) -> Self {
        Self { store: Arc::new(store) }
    }

    pub async fn store(&self, capsule_bytes: &[u8]) -> Result<String, ContentServiceError> {
        let capsule = Capsule::from_bytes(capsule_bytes)?;
        let cid = self.store.store_capsule(&capsule).await?;
        Ok(cid.to_string())
    }

    pub async fn fetch(&self, cid_str: &str) -> Result<Vec<u8>, ContentServiceError> {
        let cid: Cid = cid_str
            .parse()
            .map_err(|e: bnpm_cid::CidError| ContentServiceError::InvalidCid(e.to_string()))?;
        let capsule = self.store.get_capsule(&cid).await?;
        capsule.to_bytes().map_err(|e| ContentServiceError::CapsuleError(e.to_string()))
    }

    pub async fn has(&self, cid_str: &str) -> Result<bool, ContentServiceError> {
        let cid: Cid = cid_str
            .parse()
            .map_err(|e: bnpm_cid::CidError| ContentServiceError::InvalidCid(e.to_string()))?;
        self.store.has_capsule(&cid).await.map_err(ContentServiceError::Store)
    }

    pub fn inner(&self) -> &CasStore {
        &self.store
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContentServiceError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("capsule error: {0}")]
    CapsuleError(String),
    #[error("invalid CID: {0}")]
    InvalidCid(String),
    #[error("not found: {0}")]
    NotFound(String),
}

impl From<bnpm_capsule::CapsuleError> for ContentServiceError {
    fn from(e: bnpm_capsule::CapsuleError) -> Self {
        ContentServiceError::CapsuleError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bnpm_capsule::Capsule;

    #[tokio::test]
    async fn test_store_and_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::new(dir.path().join("cas")).await.unwrap();
        let cs = ContentService::new(store);

        let files = vec![("hello.txt".to_string(), b"Hello, World!".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let cid = capsule.cid().to_string();
        let capsule_bytes = capsule.to_bytes().unwrap();

        let stored_cid = cs.store(&capsule_bytes).await.unwrap();
        assert_eq!(stored_cid, cid);

        let fetched = cs.fetch(&stored_cid).await.unwrap();
        assert!(!fetched.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::new(dir.path().join("cas")).await.unwrap();
        let cs = ContentService::new(store);
        let result = cs.fetch("b3:doesnotexist").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_has() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::new(dir.path().join("cas")).await.unwrap();
        let cs = ContentService::new(store);

        let capsule = Capsule::pack(vec![("a.txt".to_string(), b"data".to_vec())]).unwrap();
        let cid = capsule.cid().to_string();
        let bytes = capsule.to_bytes().unwrap();

        assert!(!cs.has(&cid).await.unwrap());
        cs.store(&bytes).await.unwrap();
        assert!(cs.has(&cid).await.unwrap());
    }
}
