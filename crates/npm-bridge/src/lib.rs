pub mod federated;

use std::io::Read;

use dash_capsule::Capsule;
use dash_cid::Cid;
use dash_manifest::PackageManifest;
use flate2::read::GzDecoder;

#[derive(Debug, thiserror::Error)]
pub enum NpmBridgeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tarball extraction error: {0}")]
    Tarball(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("manifest error: {0}")]
    Manifest(#[from] dash_manifest::ManifestError),
    #[error("capsule error: {0}")]
    Capsule(#[from] dash_capsule::CapsuleError),
    #[error("CID error: {0}")]
    Cid(#[from] dash_cid::CidError),
    #[error("CBOR error: {0}")]
    Cbor(String),
    #[error("package not found: {name}")]
    PackageNotFound { name: String },
    #[error("version not found: {name}@{version}")]
    VersionNotFound { name: String, version: String },
    #[error("network error: {0}")]
    Network(String),
}

pub fn npm_registry_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("dash-npm-bridge/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("failed to create HTTP client")
}

pub async fn fetch_npm_metadata(name: &str) -> Result<serde_json::Value, NpmBridgeError> {
    let client = npm_registry_client();
    let url = format!("https://registry.npmjs.org/{}", name);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(NpmBridgeError::PackageNotFound { name: name.to_string() });
    }
    Ok(resp.json().await?)
}

pub async fn fetch_npm_tarball(name: &str, version: &str) -> Result<Vec<u8>, NpmBridgeError> {
    let client = npm_registry_client();
    let url = format!("https://registry.npmjs.org/{}/{}", name, version);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(NpmBridgeError::VersionNotFound {
            name: name.to_string(),
            version: version.to_string(),
        });
    }
    Ok(resp.bytes().await?.to_vec())
}

pub fn extract_tarball(tarball_bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, NpmBridgeError> {
    let decoder = GzDecoder::new(tarball_bytes);
    let mut archive = tar::Archive::new(decoder);
    let mut files = Vec::new();

    for entry in archive.entries().map_err(|e| NpmBridgeError::Tarball(e.to_string()))? {
        let mut entry = entry.map_err(|e| NpmBridgeError::Tarball(e.to_string()))?;
        let header = entry.header();
        if header.entry_type().is_dir() || !header.entry_type().is_file() {
            continue;
        }
        let raw_path = entry
            .path()
            .map_err(|e| NpmBridgeError::Tarball(e.to_string()))?
            .to_string_lossy()
            .to_string();
        let normalized = raw_path.strip_prefix("package/").unwrap_or(&raw_path).to_string();
        if normalized.is_empty() {
            continue;
        }
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(|e| NpmBridgeError::Tarball(e.to_string()))?;
        files.push((normalized, data));
    }

    Ok(files)
}

pub fn normalize_metadata(files: Vec<(String, Vec<u8>)>) -> Vec<(String, Vec<u8>)> {
    files
}

/// Convert npm-style package.json (string-valued dependencies) to
/// a serde_json::Value compatible with PackageManifest deserialization.
fn normalize_npm_package_json(raw: &serde_json::Value) -> serde_json::Value {
    fn convert_deps(val: &serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    if let serde_json::Value::String(version_str) = v {
                        out.insert(k.clone(), serde_json::json!({ "version": version_str }));
                    } else {
                        out.insert(k.clone(), v.clone());
                    }
                }
                serde_json::Value::Object(out)
            },
            other => other.clone(),
        }
    }

    let mut normalized = raw.clone();
    if let Some(deps) = normalized.as_object_mut() {
        let keys = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];
        for &key in &keys {
            if let Some(val) = deps.get(key) {
                deps.insert(key.to_string(), convert_deps(val));
            }
        }
    }
    normalized
}

pub async fn tarball_to_capsule(
    name: &str,
    version: &str,
    tarball_bytes: &[u8],
) -> Result<Capsule, NpmBridgeError> {
    let metadata = fetch_npm_metadata(name).await?;
    let versions = metadata
        .get("versions")
        .and_then(|v| v.as_object())
        .ok_or_else(|| NpmBridgeError::PackageNotFound { name: name.to_string() })?;
    let version_meta = versions.get(version).ok_or_else(|| NpmBridgeError::VersionNotFound {
        name: name.to_string(),
        version: version.to_string(),
    })?;

    let normalized_meta = normalize_npm_package_json(version_meta);
    let pkg_manifest: PackageManifest = serde_json::from_value(normalized_meta)?;

    let cbor_value: ciborium::Value = serde_json::from_value(serde_json::to_value(&pkg_manifest)?)
        .map_err(|e| NpmBridgeError::Cbor(e.to_string()))?;

    let manifest_cbor = dash_cid::canonical_cbor_encode(&cbor_value)?;

    let files = extract_tarball(tarball_bytes)?;
    let files = normalize_metadata(files);
    let capsule = Capsule::pack_with_manifest(manifest_cbor, files)?;
    Ok(capsule)
}

pub async fn npm_to_dash_registry(
    name: &str,
    version: &str,
) -> Result<(Capsule, Cid), NpmBridgeError> {
    let tarball = fetch_npm_tarball(name, version).await?;
    let capsule = tarball_to_capsule(name, version, &tarball).await?;
    let cid = capsule.cid();
    Ok((capsule, cid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tarball_empty() {
        let result = extract_tarball(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_metadata_identity() {
        let files = vec![("index.js".to_string(), b"content".to_vec())];
        let result = normalize_metadata(files.clone());
        assert_eq!(result, files);
    }

    #[test]
    fn test_extract_tarball_no_gzip() {
        let result = extract_tarball(b"not a valid gzip stream");
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_metadata_fails_for_nonexistent() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result =
            rt.block_on(fetch_npm_metadata("this-package-definitely-does-not-exist-12345"));
        assert!(result.is_err());
    }

    #[test]
    fn test_package_manifest_roundtrip_from_json() {
        let json = serde_json::json!({
            "name": "test-pkg",
            "version": "1.0.0",
            "description": "A test package"
        });
        let manifest: PackageManifest = serde_json::from_value(json).unwrap();
        assert_eq!(manifest.name, "test-pkg");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description.as_deref(), Some("A test package"));
    }

    #[test]
    fn test_npm_metadata_deserialize_roundtrip() {
        let json = serde_json::json!({
            "name": "test-pkg",
            "version": "1.0.0",
            "scripts": {
                "test": "echo test"
            },
            "dependencies": {
                "dep-a": "^1.0.0"
            }
        });
        let normalized = normalize_npm_package_json(&json);
        let manifest: PackageManifest = serde_json::from_value(normalized).unwrap();
        let cbor_val: ciborium::Value =
            serde_json::from_value(serde_json::to_value(&manifest).unwrap()).unwrap();
        let cbor_bytes = dash_cid::canonical_cbor_encode(&cbor_val).unwrap();
        assert!(!cbor_bytes.is_empty());
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_normalize_npm_package_json_preserves_non_deps() {
        let json = serde_json::json!({
            "name": "test",
            "version": "1.0.0",
            "description": "a",
            "scripts": { "build": "tsc" }
        });
        let normalized = normalize_npm_package_json(&json);
        assert_eq!(normalized["name"], "test");
        assert_eq!(normalized["version"], "1.0.0");
    }
}
