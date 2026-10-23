use crate::event::{Event, EventType};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    pub secret: String,
    pub events: Vec<EventType>,
    pub retry_policy: RetryPolicy,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<Duration>,
    pub active: bool,
}

impl WebhookConfig {
    pub fn new(url: impl Into<String>, secret: impl Into<String>, events: Vec<EventType>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url: url.into(),
            secret: secret.into(),
            events,
            retry_policy: RetryPolicy::default(),
            headers: None,
            timeout: Some(Duration::from_secs(30)),
            active: true,
        }
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn matches_event(&self, event_type: &EventType) -> bool {
        self.events.contains(event_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: String,
    pub event_id: Uuid,
    pub event_type: EventType,
    pub attempt: u32,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub next_retry: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("timeout")]
    Timeout,
    #[error("delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("max retries exceeded")]
    MaxRetriesExceeded,
    #[error("webhook not found: {0}")]
    NotFound(String),
    #[error("signature error: {0}")]
    SignatureError(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn sign_payload(payload: &[u8], secret: &[u8]) -> Result<String, WebhookError> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| WebhookError::SignatureError(e.to_string()))?;
    mac.update(payload);
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

pub fn verify_signature(payload: &[u8], signature: &str, secret: &[u8]) -> Result<bool, WebhookError> {
    let expected = sign_payload(payload, secret)?;
    Ok(constant_time_eq::constant_time_eq(signature.as_bytes(), expected.as_bytes()))
}

pub struct WebhookRegistry {
    webhooks: Arc<RwLock<HashMap<String, WebhookConfig>>>,
    deliveries: Arc<RwLock<HashMap<Uuid, WebhookDelivery>>>,
    http_client: Client,
}

impl WebhookRegistry {
    pub fn new() -> Result<Self, WebhookError> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            deliveries: Arc::new(RwLock::new(HashMap::new())),
            http_client,
        })
    }

    pub async fn register(&self, config: WebhookConfig) -> Result<String, WebhookError> {
        let id = config.id.clone();
        self.webhooks.write().await.insert(id.clone(), config);
        info!("Registered webhook: {}", id);
        Ok(id)
    }

    pub async fn unregister(&self, id: &str) -> Result<(), WebhookError> {
        if self.webhooks.write().await.remove(id).is_some() {
            info!("Unregistered webhook: {}", id);
            Ok(())
        } else {
            Err(WebhookError::NotFound(id.to_string()))
        }
    }

    pub async fn get(&self, id: &str) -> Option<WebhookConfig> {
        self.webhooks.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<WebhookConfig> {
        self.webhooks.read().await.values().cloned().collect()
    }

    pub async fn list_active_for_event(&self, event_type: &EventType) -> Vec<WebhookConfig> {
        self.webhooks
            .read()
            .await
            .values()
            .filter(|w| w.active && w.matches_event(event_type))
            .cloned()
            .collect()
    }

    pub async fn deliver(&self, event: &Event) -> Result<Vec<WebhookDelivery>, WebhookError> {
        let webhooks = self.list_active_for_event(&event.event_type).await;
        if webhooks.is_empty() {
            debug!("No webhooks registered for event type: {:?}", event.event_type);
            return Ok(Vec::new());
        }

        let mut deliveries = Vec::new();
        for webhook in webhooks {
            let delivery = self.deliver_to_webhook(&webhook, event).await;
            deliveries.push(delivery);
        }
        Ok(deliveries)
    }

    async fn deliver_to_webhook(&self, webhook: &WebhookConfig, event: &Event) -> WebhookDelivery {
        let delivery = WebhookDelivery {
            id: Uuid::new_v4(),
            webhook_id: webhook.id.clone(),
            event_id: event.id,
            event_type: event.event_type,
            attempt: 1,
            status: DeliveryStatus::Pending,
            response_code: None,
            response_body: None,
            error: None,
            timestamp: Utc::now(),
            next_retry: None,
        };

        self.deliveries.write().await.insert(delivery.id, delivery.clone());

        let payload = serde_json::to_vec(&event).unwrap_or_default();
        let signature = sign_payload(&payload, webhook.secret.as_bytes()).unwrap_or_default();

        let mut attempt = 1;
        let mut backoff = webhook.retry_policy.initial_backoff;
        let mut last_error = None;

        while attempt <= webhook.retry_policy.max_attempts {
            let mut req = self.http_client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .header("X-Bnpm-Event", event.event_type.as_str())
                .header("X-Bnpm-Signature", &signature)
                .header("X-Bnpm-Delivery", delivery.id.to_string())
                .body(payload.clone());

            if let Some(headers) = &webhook.headers {
                for (k, v) in headers {
                    req = req.header(k, v);
                }
            }

            let timeout_dur = webhook.timeout.unwrap_or(Duration::from_secs(30));
            let result = timeout(timeout_dur, req.send()).await;

            match result {
                Ok(Ok(response)) => {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();
                    if (200..300).contains(&status) {
                        let mut d = delivery.clone();
                        d.status = DeliveryStatus::Delivered;
                        d.response_code = Some(status);
                        d.response_body = Some(body);
                        d.attempt = attempt;
                        self.deliveries.write().await.insert(d.id, d.clone());
                        info!("Webhook {} delivered successfully (attempt {})", webhook.id, attempt);
                        return d;
                    } else {
                        last_error = Some(format!("HTTP {}: {}", status, body));
                        warn!("Webhook {} returned {} (attempt {})", webhook.id, status, attempt);
                    }
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                    error!("Webhook {} request failed (attempt {}): {}", webhook.id, attempt, e);
                }
                Err(_) => {
                    last_error = Some("timeout".into());
                    error!("Webhook {} timeout (attempt {})", webhook.id, attempt);
                }
            }

            if attempt < webhook.retry_policy.max_attempts {
                let mut d = delivery.clone();
                d.status = DeliveryStatus::Retrying;
                d.attempt = attempt;
                d.error = last_error.clone();
                d.next_retry = Some(Utc::now() + backoff);
                self.deliveries.write().await.insert(d.id, d);

                sleep(backoff).await;
                backoff = Duration::from_millis(
                    (backoff.as_millis() as f64 * webhook.retry_policy.backoff_multiplier) as u64
                ).min(webhook.retry_policy.max_backoff);
            }
            attempt += 1;
        }

        let mut d = delivery.clone();
        d.status = DeliveryStatus::Failed;
        d.error = last_error.or(Some("max retries exceeded".into()));
        d.attempt = attempt - 1;
        self.deliveries.write().await.insert(d.id, d.clone());
        error!("Webhook {} failed after {} attempts", webhook.id, d.attempt);
        d
    }

    pub async fn get_delivery(&self, id: Uuid) -> Option<WebhookDelivery> {
        self.deliveries.read().await.get(&id).cloned()
    }

    pub async fn list_deliveries(&self, limit: usize) -> Vec<WebhookDelivery> {
        let mut deliveries: Vec<_> = self.deliveries.read().await.values().cloned().collect();
        deliveries.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        deliveries.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_creation() {
        let config = WebhookConfig::new("https://example.com/webhook", "secret", vec![EventType::PackagePublished]);
        assert!(!config.id.is_empty());
        assert_eq!(config.url, "https://example.com/webhook");
        assert_eq!(config.secret, "secret");
        assert!(config.matches_event(&EventType::PackagePublished));
        assert!(!config.matches_event(&EventType::PackageYanked));
    }

    #[test]
    fn test_sign_verify_payload() {
        let payload = b"test payload";
        let secret = b"secret key";
        let signature = sign_payload(payload, secret).unwrap();
        assert!(!signature.is_empty());
        assert!(verify_signature(payload, &signature, secret).unwrap());
        assert!(!verify_signature(b"different", &signature, secret).unwrap());
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_backoff, Duration::from_secs(1));
    }
}