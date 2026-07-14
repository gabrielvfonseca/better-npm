use chrono::{DateTime, Utc};
use dash_proto::dpr_v1::Binding;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogSource {
    Local { log_index: u64 },
    Upstream { upstream_name: String, log_index: u64 },
    NpmBridge { log_index: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedLogEntry {
    pub name: String,
    pub bindings: Vec<Binding>,
    pub source: LogSource,
    pub timestamp: DateTime<Utc>,
    pub log_index: u64,
}

impl FederatedLogEntry {
    pub fn local(name: String, bindings: Vec<Binding>, log_index: u64) -> Self {
        Self {
            name,
            bindings,
            source: LogSource::Local { log_index },
            timestamp: Utc::now(),
            log_index,
        }
    }

    pub fn upstream(name: String, bindings: Vec<Binding>, upstream_name: String, log_index: u64) -> Self {
        Self {
            name,
            bindings,
            source: LogSource::Upstream { upstream_name, log_index },
            timestamp: Utc::now(),
            log_index,
        }
    }

    pub fn npm_bridge(name: String, bindings: Vec<Binding>, log_index: u64) -> Self {
        Self {
            name,
            bindings,
            source: LogSource::NpmBridge { log_index },
            timestamp: Utc::now(),
            log_index,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransparencyLogConfig {
    pub max_entries: Option<usize>,
    pub persist_path: Option<String>,
}

pub struct TransparencyLog {
    config: TransparencyLogConfig,
    entries: VecDeque<FederatedLogEntry>,
    next_log_index: u64,
}

impl TransparencyLog {
    pub fn new(config: TransparencyLogConfig) -> Self {
        Self {
            config,
            entries: VecDeque::new(),
            next_log_index: 1,
        }
    }

    pub async fn append(&mut self, entry: FederatedLogEntry) -> Result<u64, TransparencyLogError> {
        let log_index = self.next_log_index;
        let mut entry = entry;
        entry.log_index = log_index;
        self.next_log_index += 1;

        if let Some(max) = self.config.max_entries {
            while self.entries.len() >= max {
                self.entries.pop_front();
            }
        }

        self.entries.push_back(entry);
        debug!("Appended to transparency log at index {}", log_index);
        Ok(log_index)
    }

    pub fn entries(&self) -> &VecDeque<FederatedLogEntry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut VecDeque<FederatedLogEntry> {
        &mut self.entries
    }

    pub fn next_index(&self) -> u64 {
        self.next_log_index
    }

    pub fn get_entry(&self, log_index: u64) -> Option<&FederatedLogEntry> {
        self.entries.iter().find(|e| e.log_index == log_index)
    }

    pub fn get_entries_for_package(&self, name: &str) -> Vec<&FederatedLogEntry> {
        self.entries.iter().filter(|e| e.name == name).collect()
    }

    pub fn get_entries_from_upstream(&self, _upstream_name: &str) -> Vec<&FederatedLogEntry> {
        self.entries.iter()
            .filter(|e| matches!(&e.source, LogSource::Upstream { upstream_name, .. } if upstream_name == upstream_name))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub async fn persist(&self) -> Result<(), TransparencyLogError> {
        if let Some(path) = &self.config.persist_path {
            let json = serde_json::to_string_pretty(&self.entries)?;
            tokio::fs::write(path, json).await?;
        }
        Ok(())
    }

    pub async fn load(&mut self, path: &str) -> Result<(), TransparencyLogError> {
        let content = tokio::fs::read_to_string(path).await?;
        let entries: VecDeque<FederatedLogEntry> = serde_json::from_str(&content)?;
        self.next_log_index = entries.iter().map(|e| e.log_index).max().unwrap_or(0) + 1;
        self.entries = entries;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransparencyLogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Entry not found: {0}")]
    NotFound(u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_proto::dpr_v1::Binding;

    #[tokio::test]
    async fn test_transparency_log_append() {
        let config = TransparencyLogConfig::default();
        let mut log = TransparencyLog::new(config);

        let entry = FederatedLogEntry::local(
            "test-pkg".to_string(),
            vec![Binding {
                version: "1.0.0".to_string(),
                cid: "b3:abc".to_string(),
                log_index: 0,
                inclusion_proof: None,
                provenance: None,
            }],
            0,
        );

        let idx = log.append(entry).await.unwrap();
        assert_eq!(idx, 1);
        assert_eq!(log.len(), 1);
        assert_eq!(log.next_index(), 2);
    }

    #[tokio::test]
    async fn test_transparency_log_max_entries() {
        let config = TransparencyLogConfig { max_entries: Some(2), persist_path: None };
        let mut log = TransparencyLog::new(config);

        for i in 1..=5 {
            let entry = FederatedLogEntry::local(
                format!("pkg-{}", i),
                vec![Binding {
                    version: "1.0.0".to_string(),
                    cid: format!("b3:{}", i),
                    log_index: 0,
                    inclusion_proof: None,
                    provenance: None,
                }],
                0,
            );
            log.append(entry).await.unwrap();
        }

        assert_eq!(log.len(), 2);
        assert_eq!(log.next_index(), 6);
    }

    #[tokio::test]
    async fn test_transparency_log_source_types() {
        let config = TransparencyLogConfig::default();
        let mut log = TransparencyLog::new(config);

        let local_entry = FederatedLogEntry::local("pkg".into(), vec![], 1);
        let upstream_entry = FederatedLogEntry::upstream("pkg".into(), vec![], "upstream-1".into(), 2);
        let npm_entry = FederatedLogEntry::npm_bridge("pkg".into(), vec![], 3);

        log.append(local_entry).await.unwrap();
        log.append(upstream_entry).await.unwrap();
        log.append(npm_entry).await.unwrap();

        let entries: Vec<_> = log.entries().iter().collect();
        assert!(matches!(&entries[0].source, LogSource::Local { .. }));
        assert!(matches!(&entries[1].source, LogSource::Upstream { upstream_name, .. } if upstream_name == "upstream-1"));
        assert!(matches!(&entries[2].source, LogSource::NpmBridge { .. }));
    }

    #[tokio::test]
    async fn test_get_entries_for_package() {
        let config = TransparencyLogConfig::default();
        let mut log = TransparencyLog::new(config);

        log.append(FederatedLogEntry::local("pkg-a".into(), vec![], 1)).await.unwrap();
        log.append(FederatedLogEntry::local("pkg-b".into(), vec![], 2)).await.unwrap();
        log.append(FederatedLogEntry::local("pkg-a".into(), vec![], 3)).await.unwrap();

        let pkg_a = log.get_entries_for_package("pkg-a");
        assert_eq!(pkg_a.len(), 2);
    }
}