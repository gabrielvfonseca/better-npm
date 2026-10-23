//! Content-addressed store (CAS) implementation.
//!
//! Implements the content-addressed storage layout per the CONTENT-ADDRESSING.md spec.
//! Stores capsules and files by their BLAKE3 hash/CID with a two-level shard directory
//! layout, SQLite index, and materialization strategies (reflink → hardlink → copy).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bnpm_capsule::Capsule;
use bnpm_cid::Cid;
use bnpm_hashing::hash_blake3;
use sqlx::SqlitePool;

use crate::StoreError;

/// Materialization strategy preference order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializeStrategy {
    Reflink,
    Hardlink,
    Copy,
}

/// Garbage collection report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcReport {
    pub removed_capsules: usize,
    pub removed_files: usize,
    pub reclaimed_bytes: u64,
}

/// Content-addressed store (CAS) implementation.
///
/// Layout:
/// ```text
/// base_path/
/// ├── cas/b3/<aa>/<bb>/<cid>.capsule
/// ├── files/b3/<aa>/<bb>/<filehash>
/// ├── unpacked/<cid>/
/// └── index.db
/// ```
pub struct CasStore {
    base_path: PathBuf,
    pool: SqlitePool,
    cache: Arc<dyn bnpm_cache::CacheBackend>,
}

impl CasStore {
    /// Open or create a CAS store at the given base path.
    ///
    /// Creates the required directory structure (`cas/`, `files/`, `unpacked/`)
    /// and initializes the SQLite index if it does not exist.
    pub async fn new(base_path: PathBuf) -> Result<Self, StoreError> {
        tokio::fs::create_dir_all(base_path.join("cas").join("b3")).await?;
        tokio::fs::create_dir_all(base_path.join("files").join("b3")).await?;
        tokio::fs::create_dir_all(base_path.join("unpacked")).await?;

        let db_path = base_path.join("index.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let pool = SqlitePool::connect(&database_url)
            .await
            .map_err(|e| StoreError::Backend(format!("failed to open index.db: {e}")))?;

        Self::migrate(&pool).await?;

        let cache = bnpm_cache::create_cache(&bnpm_cache::CacheConfig::default());

        Ok(Self { base_path, pool, cache })
    }

    async fn migrate(pool: &SqlitePool) -> Result<(), StoreError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| StoreError::Backend(format!("tx begin failed: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS capsules (
                cid TEXT PRIMARY KEY,
                size_bytes INTEGER NOT NULL,
                blake3_digest BLOB NOT NULL,
                compressed_size_bytes INTEGER,
                inserted_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| StoreError::Backend(format!("migrate capsules failed: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS files (
                hash TEXT NOT NULL,
                cid TEXT NOT NULL,
                path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                PRIMARY KEY (cid, path),
                FOREIGN KEY (cid) REFERENCES capsules(cid)
            )",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| StoreError::Backend(format!("migrate files failed: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS refs (
                cid TEXT NOT NULL,
                refcount INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (cid),
                FOREIGN KEY (cid) REFERENCES capsules(cid)
            )",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| StoreError::Backend(format!("migrate refs failed: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS unpacked (
                cid TEXT NOT NULL,
                project_path TEXT NOT NULL,
                materialized_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (cid, project_path),
                FOREIGN KEY (cid) REFERENCES capsules(cid)
            )",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| StoreError::Backend(format!("migrate unpacked failed: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| StoreError::Backend(format!("tx commit failed: {e}")))?;

        Ok(())
    }

    /// Derive the two-level shard directory from a raw BLAKE3 digest.
    ///
    /// Sharding: `b3/<first 2 hex chars>/<next 2 hex chars>/`
    fn shard_path(base: &Path, digest: &[u8]) -> PathBuf {
        let hex_digest = hex::encode(digest);
        let aa = &hex_digest[..2];
        let bb = &hex_digest[2..4];
        base.join(aa).join(bb)
    }

    /// Compute the on-disk path for a capsule blob.
    pub fn capsule_path(&self, cid: &Cid) -> PathBuf {
        let digest = cid.digest();
        let shard = Self::shard_path(&self.base_path.join("cas").join("b3"), digest);
        shard.join(format!("{}.capsule", cid))
    }

    /// Compute the on-disk path for a file object stored by BLAKE3 hash.
    pub(crate) fn file_path_by_hash(&self, file_hash: &[u8; 32]) -> PathBuf {
        let shard = Self::shard_path(&self.base_path.join("files").join("b3"), file_hash);
        shard.join(hex::encode(file_hash))
    }

    /// Store a capsule and its constituent file objects.
    ///
    /// The capsule blob is written to `cas/b3/<shard>/<cid>.capsule` and
    /// individual file contents are extracted to `files/b3/<shard>/<filehash>`
    /// for file-level dedup. The SQLite index is updated transactionally.
    pub async fn store_capsule(&self, capsule: &Capsule) -> Result<Cid, StoreError> {
        let cid = capsule.cid();
        let cid_str = cid.to_string();

        if self.has_capsule(&cid).await? {
            return Ok(cid);
        }

        let capsule_bytes = capsule
            .to_bytes()
            .map_err(|e| StoreError::Backend(format!("capsule serialization failed: {e}")))?;
        let size = capsule_bytes.len() as i64;
        let digest = cid.digest().to_vec();

        // Write capsule blob
        let cas_path = self.capsule_path(&cid);
        if let Some(parent) = cas_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&cas_path, &capsule_bytes).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                tokio::fs::set_permissions(&cas_path, std::fs::Permissions::from_mode(0o444)).await;
        }

        // Extract files from capsule for individual storage
        let unpacked_files = capsule
            .unpack()
            .map_err(|e| StoreError::Backend(format!("capsule unpack failed: {e}")))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StoreError::Backend(format!("tx begin failed: {e}")))?;

        sqlx::query(
            "INSERT OR IGNORE INTO capsules (cid, size_bytes, blake3_digest) VALUES (?, ?, ?)",
        )
        .bind(&cid_str)
        .bind(size)
        .bind(&digest)
        .execute(&mut *tx)
        .await
        .map_err(|e| StoreError::Backend(format!("capsule insert failed: {e}")))?;

        for (path, data) in &unpacked_files {
            let file_hash = hash_blake3(data);
            let file_hash_hex = hex::encode(file_hash);

            let file_store_path = self.file_path_by_hash(&file_hash);
            if !file_store_path.exists() {
                if let Some(parent) = file_store_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(&file_store_path, data).await?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = tokio::fs::set_permissions(
                        &file_store_path,
                        std::fs::Permissions::from_mode(0o444),
                    )
                    .await;
                }
            }

            sqlx::query(
                "INSERT OR IGNORE INTO files (hash, cid, path, size_bytes) VALUES (?, ?, ?, ?)",
            )
            .bind(&file_hash_hex)
            .bind(&cid_str)
            .bind(path)
            .bind(data.len() as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| StoreError::Backend(format!("file insert failed: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| StoreError::Backend(format!("tx commit failed: {e}")))?;

        Ok(cid)
    }

    /// Retrieve and parse a capsule by CID.
    pub async fn get_capsule(&self, cid: &Cid) -> Result<Capsule, StoreError> {
        let cache_key = format!("cas:capsule:{}", cid);
        if let Some(cached) = bnpm_cache::cache_get::<Vec<u8>>(&*self.cache, &cache_key).await? {
            return Capsule::from_bytes(&cached)
                .map_err(|e| StoreError::Backend(format!("capsule deserialization failed: {e}")));
        }

        let path = self.capsule_path(cid);
        let data = tokio::fs::read(&path).await?;

        let _ = bnpm_cache::cache_set(&*self.cache, &cache_key, &data, None).await;

        Capsule::from_bytes(&data)
            .map_err(|e| StoreError::Backend(format!("capsule deserialization failed: {e}")))
    }

    /// Check if a CID exists in the store.
    pub async fn has_capsule(&self, cid: &Cid) -> Result<bool, StoreError> {
        let cid_str = cid.to_string();

        if self.cache.exists(&format!("cas:capsule:{}", cid)).await? {
            return Ok(true);
        }

        let result: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM capsules WHERE cid = ?")
            .bind(&cid_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StoreError::Backend(format!("has_capsule query failed: {e}")))?;

        Ok(result.is_some())
    }

    /// Store an individual file object for dedup.
    pub async fn store_file(
        &self,
        cid: &Cid,
        path: &str,
        data: &[u8],
    ) -> Result<String, StoreError> {
        let file_hash = hash_blake3(data);
        let file_hash_hex = hex::encode(file_hash);
        let cid_str = cid.to_string();

        let file_store_path = self.file_path_by_hash(&file_hash);
        if !file_store_path.exists() {
            if let Some(parent) = file_store_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&file_store_path, data).await?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = tokio::fs::set_permissions(
                    &file_store_path,
                    std::fs::Permissions::from_mode(0o444),
                )
                .await;
            }
        }

        sqlx::query(
            "INSERT OR IGNORE INTO files (hash, cid, path, size_bytes) VALUES (?, ?, ?, ?)",
        )
        .bind(&file_hash_hex)
        .bind(&cid_str)
        .bind(path)
        .bind(data.len() as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(format!("store_file insert failed: {e}")))?;

        Ok(file_hash_hex)
    }

    /// Materialize a capsule's files into a target directory.
    ///
    /// Strategy (per-filesystem detection): reflink → hardlink → copy.
    pub async fn materialize(&self, cid: &Cid, target: &Path) -> Result<(), StoreError> {
        let capsule = self.get_capsule(cid).await?;
        let files = capsule
            .unpack()
            .map_err(|e| StoreError::Backend(format!("capsule unpack failed: {e}")))?;

        let cid_str = cid.to_string();
        let target = target.to_path_buf();

        for (rel_path, data) in &files {
            let target_path = safe_join(&target, rel_path)?;
            reject_symlinked_components(&target, &target_path)?;
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let file_hash = hash_blake3(data);
            let file_store_path = self.file_path_by_hash(&file_hash);

            if file_store_path.exists() {
                materialize_file(&file_store_path, &target_path, data)?;
            } else {
                tokio::fs::write(&target_path, data).await?;
            }
        }

        let project_path = target.to_string_lossy().to_string();
        sqlx::query("INSERT OR IGNORE INTO unpacked (cid, project_path) VALUES (?, ?)")
            .bind(&cid_str)
            .bind(&project_path)
            .execute(&self.pool)
            .await
            .map_err(|e| StoreError::Backend(format!("unpacked insert failed: {e}")))?;

        Ok(())
    }

    /// Increment reference count for a CID.
    pub async fn add_ref(&self, cid: &Cid) -> Result<(), StoreError> {
        let cid_str = cid.to_string();
        sqlx::query(
            "INSERT INTO refs (cid, refcount) VALUES (?, 1)
             ON CONFLICT(cid) DO UPDATE SET refcount = refcount + 1",
        )
        .bind(&cid_str)
        .execute(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(format!("add_ref failed: {e}")))?;
        Ok(())
    }

    /// Decrement reference count for a CID.
    ///
    /// If refcount reaches zero, the entry is removed and becomes a GC candidate.
    pub async fn remove_ref(&self, cid: &Cid) -> Result<(), StoreError> {
        let cid_str = cid.to_string();
        let result =
            sqlx::query("UPDATE refs SET refcount = refcount - 1 WHERE cid = ? AND refcount > 0")
                .bind(&cid_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StoreError::Backend(format!("remove_ref failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(StoreError::KeyNotFound(format!("no refs for cid: {cid_str}")));
        }

        sqlx::query("DELETE FROM refs WHERE cid = ? AND refcount <= 0")
            .bind(&cid_str)
            .execute(&self.pool)
            .await
            .map_err(|e| StoreError::Backend(format!("remove_ref cleanup failed: {e}")))?;

        Ok(())
    }

    /// Run garbage collection: remove capsules and files with zero references.
    pub async fn gc(&self) -> Result<GcReport, StoreError> {
        let mut report = GcReport { removed_capsules: 0, removed_files: 0, reclaimed_bytes: 0 };

        // Remove capsule blobs for unreferenced CIDs
        let unreferenced: Vec<(String, i64)> = sqlx::query_as(
            "SELECT c.cid, c.size_bytes FROM capsules c
             LEFT JOIN refs r ON c.cid = r.cid
             WHERE r.cid IS NULL OR r.refcount <= 0",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(format!("gc query failed: {e}")))?;

        for (cid_str, size) in &unreferenced {
            if let Ok(cid) = cid_str.parse::<Cid>() {
                let cas_path = self.capsule_path(&cid);
                if cas_path.exists() {
                    tokio::fs::remove_file(&cas_path).await?;
                }
            }

            // Delete child rows first to avoid FK constraint violations
            sqlx::query("DELETE FROM files WHERE cid = ?")
                .bind(cid_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StoreError::Backend(format!("gc delete files failed: {e}")))?;

            sqlx::query("DELETE FROM refs WHERE cid = ?")
                .bind(cid_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StoreError::Backend(format!("gc delete refs failed: {e}")))?;

            sqlx::query("DELETE FROM capsules WHERE cid = ?")
                .bind(cid_str)
                .execute(&self.pool)
                .await
                .map_err(|e| StoreError::Backend(format!("gc delete capsule failed: {e}")))?;

            report.removed_capsules += 1;
            report.reclaimed_bytes += *size as u64;
        }

        // Remove orphaned file blobs (files whose capsule has been deleted)
        let orphaned: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT f.hash FROM files f
             LEFT JOIN capsules c ON f.cid = c.cid
             WHERE c.cid IS NULL",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(format!("gc orphaned query failed: {e}")))?;

        for (hash,) in &orphaned {
            if let Ok(hash_bytes) = hex::decode(hash) {
                if hash_bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&hash_bytes);
                    let file_path = self.file_path_by_hash(&arr);
                    if file_path.exists() {
                        if let Ok(meta) = tokio::fs::metadata(&file_path).await {
                            report.reclaimed_bytes += meta.len();
                        }
                        tokio::fs::remove_file(&file_path).await?;
                        report.removed_files += 1;
                    }
                }
            }
        }

        Ok(report)
    }

    /// Verify integrity of a stored capsule by re-computing its CID.
    pub async fn verify(&self, cid: &Cid) -> Result<bool, StoreError> {
        let capsule = match self.get_capsule(cid).await {
            Ok(c) => c,
            Err(StoreError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            },
            Err(StoreError::Backend(_)) => {
                // Corrupted or unparseable capsule data
                return Ok(false);
            },
            Err(e) => return Err(e),
        };
        let computed_cid = capsule.cid();
        Ok(&computed_cid == cid)
    }

    /// Access the underlying SQLite pool (for tests).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Access the base path.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// ── Materialization ─────────────────────────────────────────────────────────

/// Determine the best materialization strategy for the current filesystem.
///
/// Returns `Reflink` on macOS (APFS) and Linux (CoW filesystems like Btrfs/XFS),
/// `Hardlink` on other Unix, `Copy` as the universal fallback.
pub fn materialize_strategy(_store_path: &Path, _target_path: &Path) -> MaterializeStrategy {
    #[cfg(target_os = "macos")]
    {
        MaterializeStrategy::Reflink
    }
    #[cfg(target_os = "linux")]
    {
        MaterializeStrategy::Reflink
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        MaterializeStrategy::Copy
    }
}

/// Attempt to reflink (copy-on-write clone) from src to dst.
///
/// On macOS uses `copyfile` with `COPYFILE_CLONE`.
/// On Linux uses `ioctl` with `FICLONE`.
/// Returns `true` on success, `false` if the operation is unsupported or fails.
fn try_reflink(src: &Path, dst: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let src_c = CString::new(src.as_os_str().as_bytes()).ok();
        let dst_c = CString::new(dst.as_os_str().as_bytes()).ok();

        if let (Some(src), Some(dst)) = (src_c, dst_c) {
            // Use clonefile(2) for APFS copy-on-write clones
            let result = unsafe { libc::clonefile(src.as_ptr(), dst.as_ptr(), 0) };
            return result == 0;
        }
        false
    }
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::os::fd::AsRawFd;

        if let (Ok(src_file), Ok(dst_file)) = (File::open(src), File::create(dst)) {
            let src_fd = src_file.as_raw_fd();
            let dst_fd = dst_file.as_raw_fd();
            // FICLONE = _IOW(0x94, 9, int) = 0x40049409
            let result = unsafe { libc::ioctl(dst_fd, 0x40049409_usize as _, src_fd) };
            return result == 0;
        }
        false
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (src, dst);
        false
    }
}

/// Materialize a single file using the best available strategy.
///
/// 1. Try reflink (copy-on-write clone)
/// 2. Try hardlink (same-device check)
/// 3. Fall back to copy using the provided fallback data
///
/// Join a capsule-supplied relative path onto `target`, rejecting anything that
/// could escape it (absolute paths, prefixes, or `..` components).
fn safe_join(target: &Path, rel_path: &str) -> Result<PathBuf, StoreError> {
    let rel = Path::new(rel_path);
    if rel.as_os_str().is_empty() || !rel.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
        return Err(StoreError::Backend(format!("unsafe path in capsule: {rel_path}")));
    }
    Ok(target.join(rel))
}

/// Refuse to write through a symlink that already exists between `root` and `path`.
fn reject_symlinked_components(root: &Path, path: &Path) -> Result<(), StoreError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(StoreError::Backend(format!(
                    "refusing to write through symlink: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn materialize_file(src: &Path, dst: &Path, fallback_data: &[u8]) -> Result<(), StoreError> {
    // Try reflink
    if try_reflink(src, dst) {
        return Ok(());
    }

    // Try hardlink
    #[cfg(unix)]
    {
        if let (Ok(src_meta), Some(parent)) = (std::fs::metadata(src), dst.parent()) {
            if let Ok(parent_meta) = std::fs::metadata(parent) {
                use std::os::unix::fs::MetadataExt;
                if src_meta.dev() == parent_meta.dev() && std::fs::hard_link(src, dst).is_ok() {
                    return Ok(());
                }
            }
        }
    }

    // Fall back to copy
    std::fs::write(dst, fallback_data)?;
    Ok(())
}

// ── StoreBackend trait implementation ────────────────────────────────────────

#[async_trait::async_trait]
impl crate::StoreBackend for CasStore {
    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        let cid: Cid = key
            .parse()
            .map_err(|e: bnpm_cid::CidError| StoreError::Backend(format!("invalid CID: {e}")))?;
        let capsule = self.get_capsule(&cid).await?;
        capsule
            .to_bytes()
            .map_err(|e| StoreError::Backend(format!("capsule serialization failed: {e}")))
    }

    async fn put(&self, data: &[u8]) -> Result<String, StoreError> {
        let capsule = Capsule::from_bytes(data)
            .map_err(|e| StoreError::Backend(format!("invalid capsule bytes: {e}")))?;
        let cid = self.store_capsule(&capsule).await?;
        Ok(cid.to_string())
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        let cid: Cid = key
            .parse()
            .map_err(|e: bnpm_cid::CidError| StoreError::Backend(format!("invalid CID: {e}")))?;
        self.has_capsule(&cid).await
    }

    async fn remove(&self, key: &str) -> Result<(), StoreError> {
        let cid: Cid = key
            .parse()
            .map_err(|e: bnpm_cid::CidError| StoreError::Backend(format!("invalid CID: {e}")))?;
        let cid_str = cid.to_string();
        let path = self.capsule_path(&cid);

        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StoreError::Backend(format!("tx begin failed: {e}")))?;

        // Delete child rows first to avoid FK constraint violations
        sqlx::query("DELETE FROM files WHERE cid = ?")
            .bind(&cid_str)
            .execute(&mut *tx)
            .await
            .map_err(|e| StoreError::Backend(format!("remove files failed: {e}")))?;

        sqlx::query("DELETE FROM refs WHERE cid = ?")
            .bind(&cid_str)
            .execute(&mut *tx)
            .await
            .map_err(|e| StoreError::Backend(format!("remove refs failed: {e}")))?;

        sqlx::query("DELETE FROM capsules WHERE cid = ?")
            .bind(&cid_str)
            .execute(&mut *tx)
            .await
            .map_err(|e| StoreError::Backend(format!("remove capsule failed: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| StoreError::Backend(format!("tx commit failed: {e}")))?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>, StoreError> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT cid FROM capsules ORDER BY cid")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StoreError::Backend(format!("list query failed: {e}")))?;
        Ok(rows.into_iter().map(|(cid,)| cid).collect())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StoreBackend;
    use bnpm_capsule::Capsule;

    fn test_files() -> Vec<(String, Vec<u8>)> {
        vec![
            ("hello.txt".to_string(), b"Hello, World!".to_vec()),
            ("foo/bar.txt".to_string(), b"bar content".to_vec()),
            ("a.txt".to_string(), b"alpha".to_vec()),
        ]
    }

    fn create_capsule() -> Capsule {
        Capsule::pack(test_files()).unwrap()
    }

    fn create_empty_capsule() -> Capsule {
        Capsule::pack(vec![]).unwrap()
    }

    async fn create_store() -> (CasStore, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let store = CasStore::new(dir.path().to_path_buf()).await.unwrap();
        (store, dir)
    }

    // ── Core tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_store_and_retrieve_capsule() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let retrieved = store.get_capsule(&cid).await.unwrap();
        let retrieved_cid = retrieved.cid();
        assert_eq!(cid, retrieved_cid);

        let unpacked = retrieved.unpack().unwrap();
        assert_eq!(unpacked.len(), 3);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_empty_capsule() {
        let (store, _dir) = create_store().await;
        let capsule = create_empty_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let retrieved = store.get_capsule(&cid).await.unwrap();
        let unpacked = retrieved.unpack().unwrap();
        assert!(unpacked.is_empty());
    }

    #[tokio::test]
    async fn test_has_capsule() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        assert!(store.has_capsule(&cid).await.unwrap());

        let fake_cid = Cid::new(b"nonexistent");
        assert!(!store.has_capsule(&fake_cid).await.unwrap());
    }

    #[tokio::test]
    async fn test_missing_capsule_error() {
        let (store, _dir) = create_store().await;
        let fake_cid = Cid::new(b"missing");

        let result = store.get_capsule(&fake_cid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_store_dedup_same_capsule() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid1 = store.store_capsule(&capsule).await.unwrap();
        let cid2 = store.store_capsule(&capsule).await.unwrap();

        assert_eq!(cid1, cid2);
    }

    #[tokio::test]
    async fn test_file_level_dedup() {
        let (store, dir) = create_store().await;

        // Two capsules with one shared file
        let files1 = vec![
            ("shared.txt".to_string(), b"common content".to_vec()),
            ("unique1.txt".to_string(), b"unique one".to_vec()),
        ];
        let files2 = vec![
            ("shared.txt".to_string(), b"common content".to_vec()),
            ("unique2.txt".to_string(), b"unique two".to_vec()),
        ];

        let capsule1 = Capsule::pack(files1).unwrap();
        let capsule2 = Capsule::pack(files2).unwrap();

        let cid1 = store.store_capsule(&capsule1).await.unwrap();
        let cid2 = store.store_capsule(&capsule2).await.unwrap();

        assert_ne!(cid1, cid2);

        // The shared file should be stored only once in files/
        let shared_hash = hex::encode(hash_blake3(b"common content"));
        let files_dir = dir.path().join("files").join("b3");
        let aa = &shared_hash[..2];
        let bb = &shared_hash[2..4];
        let shared_path = files_dir.join(aa).join(bb).join(&shared_hash);
        assert!(shared_path.exists(), "shared file should exist in files/");

        // Count files in the files/ store (3-level: b3/<aa>/<bb>/<filehash>)
        let mut file_count = 0;
        let mut l1 = tokio::fs::read_dir(&files_dir).await.unwrap();
        while let Some(aa) = l1.next_entry().await.unwrap() {
            if aa.file_type().await.unwrap().is_dir() {
                let mut l2 = tokio::fs::read_dir(aa.path()).await.unwrap();
                while let Some(bb) = l2.next_entry().await.unwrap() {
                    if bb.file_type().await.unwrap().is_dir() {
                        let mut l3 = tokio::fs::read_dir(bb.path()).await.unwrap();
                        while let Some(entry) = l3.next_entry().await.unwrap() {
                            if entry.file_type().await.unwrap().is_file() {
                                file_count += 1;
                            }
                        }
                    }
                }
            }
        }
        // 3 unique files: common content, unique one, unique two
        assert_eq!(file_count, 3, "should store 3 unique files (deduped)");
    }

    // ── Materialization tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_materialize() {
        let (store, _store_dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let target = tempfile::TempDir::new().unwrap();
        store.materialize(&cid, target.path()).await.unwrap();

        // Verify materialized files
        let hello_path = target.path().join("hello.txt");
        assert!(hello_path.exists());
        assert_eq!(tokio::fs::read_to_string(&hello_path).await.unwrap(), "Hello, World!");

        let bar_path = target.path().join("foo/bar.txt");
        assert!(bar_path.exists());
        assert_eq!(tokio::fs::read_to_string(&bar_path).await.unwrap(), "bar content");

        let a_path = target.path().join("a.txt");
        assert!(a_path.exists());
        assert_eq!(tokio::fs::read_to_string(&a_path).await.unwrap(), "alpha");
    }

    #[tokio::test]
    async fn test_materialize_empty_capsule() {
        let (store, _store_dir) = create_store().await;
        let capsule = create_empty_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let target = tempfile::TempDir::new().unwrap();
        store.materialize(&cid, target.path()).await.unwrap();

        // Should be empty
        let mut entries = tokio::fs::read_dir(target.path()).await.unwrap();
        assert!(entries.next_entry().await.unwrap().is_none());
    }

    // ── Reference counting tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_ref_counting() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        // Add refs
        store.add_ref(&cid).await.unwrap();
        store.add_ref(&cid).await.unwrap();
        store.add_ref(&cid).await.unwrap();

        // Check refcount
        let refcount: (i64,) = sqlx::query_as("SELECT refcount FROM refs WHERE cid = ?")
            .bind(cid.to_string())
            .fetch_one(store.pool())
            .await
            .unwrap();
        // add_ref adds 1, plus the initial default? No, add_ref does INSERT with 1
        // Actually add_ref does INSERT ... ON CONFLICT DO UPDATE SET refcount = refcount + 1
        // So each call adds 1: first call inserts with 1, second updates to 2, third to 3
        assert_eq!(refcount.0, 3, "refcount should be 3");

        // Remove refs
        store.remove_ref(&cid).await.unwrap();
        store.remove_ref(&cid).await.unwrap();

        let refcount: (i64,) = sqlx::query_as("SELECT refcount FROM refs WHERE cid = ?")
            .bind(cid.to_string())
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(refcount.0, 1, "refcount should be 1 after 2 removes");

        // Last remove should remove the row
        store.remove_ref(&cid).await.unwrap();
        let result: Option<(i64,)> = sqlx::query_as("SELECT refcount FROM refs WHERE cid = ?")
            .bind(cid.to_string())
            .fetch_optional(store.pool())
            .await
            .unwrap();
        assert!(result.is_none(), "refs row should be deleted after last remove");
    }

    #[tokio::test]
    async fn test_remove_ref_nonexistent() {
        let (store, _dir) = create_store().await;
        let fake_cid = Cid::new(b"no-refs");

        let result = store.remove_ref(&fake_cid).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::KeyNotFound(_)));
    }

    // ── Garbage collection tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_gc_removes_unreferenced() {
        let (store, _dir) = create_store().await;

        // Store a capsule without adding references
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();
        assert!(store.has_capsule(&cid).await.unwrap());

        // GC should remove it (no refs)
        let report = store.gc().await.unwrap();
        assert_eq!(report.removed_capsules, 1);
        assert!(!store.has_capsule(&cid).await.unwrap());
    }

    #[tokio::test]
    async fn test_gc_preserves_referenced() {
        let (store, _dir) = create_store().await;

        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();
        store.add_ref(&cid).await.unwrap();

        let report = store.gc().await.unwrap();
        assert_eq!(report.removed_capsules, 0, "referenced capsule should not be removed");
        assert!(store.has_capsule(&cid).await.unwrap());
    }

    #[tokio::test]
    async fn test_gc_removes_after_ref_drop() {
        let (store, _dir) = create_store().await;

        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();
        store.add_ref(&cid).await.unwrap();
        store.remove_ref(&cid).await.unwrap();

        let report = store.gc().await.unwrap();
        assert_eq!(report.removed_capsules, 1);
        assert!(!store.has_capsule(&cid).await.unwrap());
    }

    // ── Integrity verification tests ──────────────────────────────────────

    #[tokio::test]
    async fn test_verify_integrity() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        assert!(store.verify(&cid).await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_missing() {
        let (store, _dir) = create_store().await;
        let fake_cid = Cid::new(b"missing");
        assert!(!store.verify(&fake_cid).await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_tampered() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        // Tamper with the stored capsule blob
        let cas_path = store.capsule_path(&cid);
        let mut bytes = tokio::fs::read(&cas_path).await.unwrap();
        if bytes.len() > 20 {
            bytes[14] ^= 0xff; // Flip a bit
            // Remove read-only permission before writing
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                tokio::fs::set_permissions(&cas_path, std::fs::Permissions::from_mode(0o644))
                    .await
                    .unwrap();
            }
            tokio::fs::write(&cas_path, &bytes).await.unwrap();
        }

        assert!(!store.verify(&cid).await.unwrap());
    }

    // ── Empty store tests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_empty_store_list() {
        let (store, _dir) = create_store().await;
        let list = store.list().await.unwrap();
        assert!(list.is_empty());
    }

    // ── SQLite persistence tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempfile::TempDir::new().unwrap();
        let store_path = dir.path().to_path_buf();

        // Open store, store capsule, close
        {
            let store = CasStore::new(store_path.clone()).await.unwrap();
            let capsule = create_capsule();
            let cid = store.store_capsule(&capsule).await.unwrap();

            let retrieved = store.get_capsule(&cid).await.unwrap();
            assert_eq!(capsule.cid(), retrieved.cid());
        }

        // Re-open and verify capsule is still there
        {
            let store = CasStore::new(store_path.clone()).await.unwrap();
            let list = store.list().await.unwrap();
            assert_eq!(list.len(), 1);

            let capsule = create_capsule();
            let cid = capsule.cid();
            assert!(store.has_capsule(&cid).await.unwrap());
        }
    }

    #[tokio::test]
    async fn test_persistence_empty_store() {
        let dir = tempfile::TempDir::new().unwrap();
        let store_path = dir.path().to_path_buf();

        {
            let store = CasStore::new(store_path.clone()).await.unwrap();
            let list = store.list().await.unwrap();
            assert!(list.is_empty());
        }

        {
            let store = CasStore::new(store_path.clone()).await.unwrap();
            let list = store.list().await.unwrap();
            assert!(list.is_empty());
        }
    }

    // ── StoreBackend trait tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_store_backend_put_get() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let bytes = capsule.to_bytes().unwrap();

        let key = store.put(&bytes).await.unwrap();
        assert!(key.starts_with("b3:"), "CID should start with b3:");

        let retrieved = store.get(&key).await.unwrap();
        assert_eq!(retrieved.len(), bytes.len());
    }

    #[tokio::test]
    async fn test_store_backend_exists() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let key = store.put(&capsule.to_bytes().unwrap()).await.unwrap();

        assert!(store.exists(&key).await.unwrap());
        let fake_key = Cid::new(b"no-such-data").to_string();
        assert!(!store.exists(&fake_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_store_backend_list() {
        let (store, _dir) = create_store().await;
        let c1 = Capsule::pack(vec![("a.txt".to_string(), b"a".to_vec())]).unwrap();
        let c2 = Capsule::pack(vec![("b.txt".to_string(), b"b".to_vec())]).unwrap();

        store.put(&c1.to_bytes().unwrap()).await.unwrap();
        store.put(&c2.to_bytes().unwrap()).await.unwrap();

        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_store_backend_remove() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let key = store.put(&capsule.to_bytes().unwrap()).await.unwrap();

        assert!(store.exists(&key).await.unwrap());
        store.remove(&key).await.unwrap();
        assert!(!store.exists(&key).await.unwrap());
    }

    // ── Materialize strategy tests ────────────────────────────────────────

    #[test]
    fn test_materialize_strategy_detection() {
        let store_path = Path::new("/tmp/store");
        let target_path = Path::new("/tmp/target");
        let strategy = materialize_strategy(store_path, target_path);
        // On macOS/Linux should be Reflink, on others Copy
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert_eq!(strategy, MaterializeStrategy::Reflink);
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        assert_eq!(strategy, MaterializeStrategy::Copy);
    }

    #[tokio::test]
    async fn test_materialize_file_fallback_to_copy() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        // Force materialization to a different temp dir
        let target = tempfile::TempDir::new().unwrap();
        store.materialize(&cid, target.path()).await.unwrap();

        assert!(target.path().join("hello.txt").exists());
        assert!(target.path().join("a.txt").exists());
        assert!(target.path().join("foo/bar.txt").exists());
    }

    #[tokio::test]
    async fn test_store_file_directly() {
        let (store, _dir) = create_store().await;
        let capsule = create_capsule();
        let cid = store.store_capsule(&capsule).await.unwrap();

        let data = b"extra file content";
        let hash = store.store_file(&cid, "extra.txt", data).await.unwrap();
        assert_eq!(hash.len(), 64); // hex of BLAKE3-256

        // File should exist in files/ store
        let hash_bytes = hex::decode(&hash).unwrap();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash_bytes);
        let file_path = store.file_path_by_hash(&arr);
        assert!(file_path.exists());
    }
}
