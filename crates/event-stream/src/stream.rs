use crate::event::{Event, EventType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub max_subscriptions_per_client: usize,
    pub event_buffer_size: usize,
    pub ping_interval: std::time::Duration,
    pub connection_timeout: std::time::Duration,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_subscriptions_per_client: 100,
            event_buffer_size: 1000,
            ping_interval: std::time::Duration::from_secs(30),
            connection_timeout: std::time::Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub namespaces: Option<Vec<String>>,
    pub packages: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub min_severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub id: Uuid,
    pub client_id: String,
    pub event_types: Vec<EventType>,
    pub created_at: DateTime<Utc>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub filter: Option<EventFilter>,
}

impl EventSubscription {
    pub fn new(client_id: String, event_types: Vec<EventType>) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            event_types,
            created_at: Utc::now(),
            last_event_at: None,
            filter: None,
        }
    }

    pub fn matches(&self, event: &Event) -> bool {
        if !self.event_types.contains(&event.event_type) {
            return false;
        }

        if let Some(filter) = &self.filter {
            match &event.payload {
                crate::event::EventPayload::PackagePublished(p) => {
                    if let Some(namespaces) = &filter.namespaces {
                        if !namespaces.contains(&p.namespace) {
                            return false;
                        }
                    }
                    if let Some(packages) = &filter.packages {
                        if !packages.contains(&p.package) {
                            return false;
                        }
                    }
                    if let Some(publishers) = &filter.publishers {
                        if !publishers.contains(&p.publisher) {
                            return false;
                        }
                    }
                }
                crate::event::EventPayload::PolicyViolation(p) => {
                    if let Some(min_sev) = &filter.min_severity {
                        let severity_order = ["low", "medium", "high", "critical"];
                        let event_idx = severity_order.iter().position(|&s| s == p.severity.to_lowercase()).unwrap_or(0);
                        let min_idx = severity_order.iter().position(|&s| s == min_sev.to_lowercase()).unwrap_or(0);
                        if event_idx < min_idx {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }

        true
    }

    pub fn with_filter(mut self, filter: EventFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

pub struct EventStream {
    config: StreamConfig,
    event_tx: broadcast::Sender<Event>,
    subscriptions: Arc<RwLock<HashMap<Uuid, EventSubscription>>>,
    client_subscriptions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl EventStream {
    pub fn new(config: StreamConfig) -> Self {
        let (event_tx, _) = broadcast::channel(config.event_buffer_size);
        Self {
            config,
            event_tx,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            client_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    #[allow(clippy::result_large_err)]
    pub async fn publish(&self, event: Event) -> Result<usize, StreamError> {
        let sent = self.event_tx.send(event.clone())?;
        debug!("Published event {} to {} subscribers", event.id, sent);
        Ok(sent)
    }

    #[allow(clippy::result_large_err)]
    pub async fn create_subscription(
        &self,
        client_id: String,
        event_types: Vec<EventType>,
        filter: Option<EventFilter>,
    ) -> Result<EventSubscription, StreamError> {
        let mut subs = self.subscriptions.write().await;
        let mut client_subs = self.client_subscriptions.write().await;

        let client_list = client_subs.entry(client_id.clone()).or_default();
        if client_list.len() >= self.config.max_subscriptions_per_client {
            return Err(StreamError::MaxSubscriptionsExceeded);
        }

        let mut subscription = EventSubscription::new(client_id.clone(), event_types);
        if let Some(f) = filter {
            subscription = subscription.with_filter(f);
        }

        client_list.push(subscription.id);
        subs.insert(subscription.id, subscription.clone());

        info!("Created subscription {} for client {}", subscription.id, client_id);
        Ok(subscription)
    }

    #[allow(clippy::result_large_err)]
    pub async fn remove_subscription(&self, subscription_id: Uuid) -> Result<(), StreamError> {
        let mut subs = self.subscriptions.write().await;
        let mut client_subs = self.client_subscriptions.write().await;

        if let Some(sub) = subs.remove(&subscription_id) {
            if let Some(client_list) = client_subs.get_mut(&sub.client_id) {
                client_list.retain(|id| *id != subscription_id);
                if client_list.is_empty() {
                    client_subs.remove(&sub.client_id);
                }
            }
            info!("Removed subscription {}", subscription_id);
            Ok(())
        } else {
            Err(StreamError::SubscriptionNotFound(subscription_id))
        }
    }

    pub async fn get_subscription(&self, subscription_id: Uuid) -> Option<EventSubscription> {
        self.subscriptions.read().await.get(&subscription_id).cloned()
    }

    pub async fn list_client_subscriptions(&self, client_id: &str) -> Vec<EventSubscription> {
        let client_subs = self.client_subscriptions.read().await;
        let subs = self.subscriptions.read().await;
        client_subs.get(client_id)
            .map(|ids| ids.iter().filter_map(|id| subs.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    pub async fn get_stats(&self) -> StreamStats {
        let subs = self.subscriptions.read().await;
        let client_subs = self.client_subscriptions.read().await;
        StreamStats {
            total_subscriptions: subs.len(),
            total_clients: client_subs.len(),
            buffer_size: self.config.event_buffer_size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub total_subscriptions: usize,
    pub total_clients: usize,
    pub buffer_size: usize,
}

#[derive(Debug, thiserror::Error)]
#[allow(clippy::large_enum_variant)]
pub enum StreamError {
    #[error("broadcast send error: {0}")]
    BroadcastSend(#[from] broadcast::error::SendError<Event>),
    #[error("subscription not found: {0}")]
    SubscriptionNotFound(Uuid),
    #[error("max subscriptions per client exceeded")]
    MaxSubscriptionsExceeded,
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventPayload, PackagePublishedPayload, EventSource};

    #[tokio::test]
    async fn test_event_stream_publish_subscribe() {
        let config = StreamConfig::default();
        let stream = EventStream::new(config);

        let mut rx = stream.subscribe();
        let source = EventSource {
            registry: "test".to_string(),
            instance_id: "test-1".to_string(),
        };
        let event = Event::new(
            EventType::PackagePublished,
            EventPayload::PackagePublished(PackagePublishedPayload {
                package: "test".to_string(),
                version: "1.0.0".to_string(),
                cid: "b3:abc".to_string(),
                publisher: "did:bnpm:test".to_string(),
                namespace: "test".to_string(),
            }),
            source,
        );

        stream.publish(event.clone()).await.unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event.id);
    }

    #[tokio::test]
    async fn test_subscription_filter() {
        let config = StreamConfig::default();
        let stream = EventStream::new(config);

        let filter = EventFilter {
            namespaces: Some(vec!["@acme".to_string()]),
            packages: None,
            publishers: None,
            min_severity: None,
        };

        let sub = stream.create_subscription(
            "client-1".to_string(),
            vec![EventType::PackagePublished],
            Some(filter),
        ).await.unwrap();

        let source = EventSource {
            registry: "test".to_string(),
            instance_id: "test-1".to_string(),
        };

        let matching = Event::new(
            EventType::PackagePublished,
            EventPayload::PackagePublished(PackagePublishedPayload {
                package: "pkg".to_string(),
                version: "1.0.0".to_string(),
                cid: "b3:abc".to_string(),
                publisher: "did:bnpm:test".to_string(),
                namespace: "@acme".to_string(),
            }),
            source.clone(),
        );

        let non_matching = Event::new(
            EventType::PackagePublished,
            EventPayload::PackagePublished(PackagePublishedPayload {
                package: "pkg".to_string(),
                version: "1.0.0".to_string(),
                cid: "b3:abc".to_string(),
                publisher: "did:bnpm:test".to_string(),
                namespace: "@other".to_string(),
            }),
            source,
        );

        assert!(sub.matches(&matching));
        assert!(!sub.matches(&non_matching));
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let config = StreamConfig::default();
        let stream = EventStream::new(config);

        let sub1 = stream.create_subscription("client-1".to_string(), vec![EventType::PackagePublished], None).await.unwrap();
        let sub2 = stream.create_subscription("client-1".to_string(), vec![EventType::PackageYanked], None).await.unwrap();

        let client_subs = stream.list_client_subscriptions("client-1").await;
        assert_eq!(client_subs.len(), 2);

        stream.remove_subscription(sub1.id).await.unwrap();
        let client_subs = stream.list_client_subscriptions("client-1").await;
        assert_eq!(client_subs.len(), 1);
    }
}