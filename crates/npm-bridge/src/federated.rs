use std::collections::BTreeMap;

use semver::Version;

use crate::{NpmBridgeError, fetch_npm_metadata};

#[derive(Debug, Clone)]
pub struct FederatedRegistry {
    pub registry_url: String,
    pub priority: u32,
}

impl FederatedRegistry {
    pub fn new(url: impl Into<String>, priority: u32) -> Self {
        Self { registry_url: url.into(), priority }
    }

    pub fn npm_default() -> Self {
        Self { registry_url: "https://registry.npmjs.org".to_string(), priority: 10 }
    }

    pub async fn resolve_npm(&self, name: &str) -> Result<Vec<Version>, NpmBridgeError> {
        let metadata = fetch_npm_metadata(name).await?;
        let versions = metadata
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| NpmBridgeError::PackageNotFound { name: name.to_string() })?;
        let mut result: Vec<Version> = Vec::new();
        for ver_str in versions.keys() {
            if let Ok(v) = Version::parse(ver_str) {
                result.push(v);
            }
        }
        result.sort();
        Ok(result)
    }

    pub async fn resolve_npm_in_range(
        &self,
        name: &str,
        range: &semver::VersionReq,
    ) -> Result<Vec<Version>, NpmBridgeError> {
        let all_versions = self.resolve_npm(name).await?;
        Ok(all_versions.into_iter().filter(|v| range.matches(v)).collect())
    }

    pub async fn to_dash_index(
        &self,
        name: &str,
    ) -> Result<Vec<dash_resolver::RegistryIndexEntry>, NpmBridgeError> {
        let metadata = fetch_npm_metadata(name).await?;
        let versions = metadata
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| NpmBridgeError::PackageNotFound { name: name.to_string() })?;

        let mut entries = Vec::new();
        for (ver_str, ver_data) in versions {
            let deps = ver_data
                .get("dependencies")
                .and_then(|d| d.as_object())
                .map(|d| {
                    d.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("*").to_string()))
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default();

            let dev_deps = ver_data
                .get("devDependencies")
                .and_then(|d| d.as_object())
                .map(|d| {
                    d.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("*").to_string()))
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default();

            let entry = dash_resolver::RegistryIndexEntry {
                name: name.to_string(),
                version: ver_str.clone(),
                cid: None,
                yanked: false,
                dependencies: deps,
                dev_dependencies: dev_deps,
                optional_dependencies: BTreeMap::new(),
                peer_dependencies: BTreeMap::new(),
                tarball: Some(format!(
                    "{}/{}/{}",
                    self.registry_url.trim_end_matches('/'),
                    name,
                    ver_str,
                )),
                integrity: None,
            };
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_federated_registry_new() {
        let reg = FederatedRegistry::npm_default();
        assert_eq!(reg.registry_url, "https://registry.npmjs.org");
        assert_eq!(reg.priority, 10);
    }

    #[tokio::test]
    async fn test_resolve_nonexistent_package() {
        let reg = FederatedRegistry::npm_default();
        let result = reg.resolve_npm("this-package-does-not-exist-99999").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_federated_registry_priority() {
        let reg = FederatedRegistry::new("https://custom.registry", 5);
        assert_eq!(reg.priority, 5);
    }
}
