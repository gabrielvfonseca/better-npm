//! Authentication and token management for registry access.
//!
//! Supports JWT-based authentication, OAuth2 flows, and
//! basic token-based auth for Bnpm registry operations.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Error type for authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("authorization denied: {action} requires {permission}")]
    AuthorizationDenied { action: String, permission: String },
    #[error("provider error: {0}")]
    Provider(String),
    #[error("session expired")]
    SessionExpired,
}

/// Supported authentication providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthProvider {
    /// JSON Web Token based auth.
    Jwt,
    /// OAuth 2.0.
    OAuth2,
    /// Basic username/password auth.
    Basic,
}

impl AuthProvider {
    /// Return the provider as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthProvider::Jwt => "jwt",
            AuthProvider::OAuth2 => "oauth2",
            AuthProvider::Basic => "basic",
        }
    }
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An authenticated session with permissions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthSession {
    /// Authenticated user identifier.
    pub user: String,
    /// The authentication token.
    pub token: String,
    /// Token expiration timestamp.
    pub expires_at: SystemTime,
    /// Set of granted permissions.
    pub permissions: HashSet<String>,
    /// The provider used for authentication.
    pub provider: AuthProvider,
    /// Session creation time.
    pub created_at: SystemTime,
}

impl AuthSession {
    /// Check whether this session is expired.
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    /// Check whether this session has a specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Check whether this session has any of the listed permissions.
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.permissions.contains(*p))
    }

    /// Check whether this session has all of the listed permissions.
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.permissions.contains(*p))
    }

    /// Return the time until this session expires.
    pub fn ttl(&self) -> Duration {
        self.expires_at
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO)
    }
}

/// Token storage backend.
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait TokenStore: Send + Sync {
    /// Store a token for a user.
    async fn store_token(&self, user: &str, token: &str, expires_at: SystemTime) -> Result<(), AuthError>;
    /// Retrieve a token for a user.
    async fn get_token(&self, user: &str) -> Result<Option<String>, AuthError>;
    /// Remove a token for a user.
    async fn remove_token(&self, user: &str) -> Result<(), AuthError>;
}

/// In-memory token store.
pub struct MemoryTokenStore {
    tokens: Arc<dashmap::DashMap<String, (String, SystemTime)>>,
}

impl MemoryTokenStore {
    /// Create a new in-memory token store.
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(dashmap::DashMap::new()),
        }
    }
}

impl Default for MemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TokenStore for MemoryTokenStore {
    async fn store_token(&self, user: &str, token: &str, expires_at: SystemTime) -> Result<(), AuthError> {
        self.tokens.insert(user.to_string(), (token.to_string(), expires_at));
        Ok(())
    }

    async fn get_token(&self, user: &str) -> Result<Option<String>, AuthError> {
        Ok(self.tokens.get(user).map(|e| e.0.clone()))
    }

    async fn remove_token(&self, user: &str) -> Result<(), AuthError> {
        self.tokens.remove(user);
        Ok(())
    }
}

/// The authenticator handles token validation and session management.
pub struct Authenticator {
    provider: AuthProvider,
    jwt_secret: Option<String>,
    #[allow(dead_code)]
    store: Arc<dyn TokenStore>,
    allowed_permissions: HashSet<String>,
}

impl Authenticator {
    /// Create a new authenticator with a JWT secret and token store.
    pub fn new(
        provider: AuthProvider,
        jwt_secret: Option<String>,
        store: Arc<dyn TokenStore>,
    ) -> Self {
        Self {
            provider,
            jwt_secret,
            store,
            allowed_permissions: HashSet::new(),
        }
    }

    /// Set the allowed permissions.
    pub fn with_permissions(mut self, permissions: HashSet<String>) -> Self {
        self.allowed_permissions = permissions;
        self
    }

    /// Authenticate a token and return a session.
    pub async fn authenticate(&self, token: &str) -> Result<AuthSession, AuthError> {
        match self.provider {
            AuthProvider::Jwt => self.authenticate_jwt(token).await,
            AuthProvider::Basic => self.authenticate_basic(token).await,
            AuthProvider::OAuth2 => self.authenticate_oauth2(token).await,
        }
    }

    /// Authorize a session for a specific action.
    pub async fn authorize(&self, session: &AuthSession, action: &str) -> Result<bool, AuthError> {
        if session.is_expired() {
            return Err(AuthError::SessionExpired);
        }

        // Check if the session has the required permission for the action
        let permission = action_to_permission(action);
        if !session.has_permission(&permission) {
            return Err(AuthError::AuthorizationDenied {
                action: action.to_string(),
                permission,
            });
        }

        Ok(true)
    }

    /// Validate a raw token string without creating a full session.
    pub async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        match self.provider {
            AuthProvider::Jwt => {
                if let Some(secret) = &self.jwt_secret {
                    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
                    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
                    jsonwebtoken::decode::<serde_json::Value>(token, &key, &validation)
                        .map(|_| true)
                        .map_err(|e| AuthError::InvalidToken(e.to_string()))
                } else {
                    Err(AuthError::AuthenticationFailed("no JWT secret configured".to_string()))
                }
            }
            AuthProvider::Basic => Ok(!token.is_empty()),
            AuthProvider::OAuth2 => Ok(!token.is_empty()),
        }
    }

    async fn authenticate_jwt(&self, token: &str) -> Result<AuthSession, AuthError> {
        if let Some(secret) = &self.jwt_secret {
            let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
            let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

            let token_data = jsonwebtoken::decode::<serde_json::Value>(token, &key, &validation)
                .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

            let claims = token_data.claims;
            let user = claims
                .get("sub")
                .and_then(|v| v.as_str())
                .unwrap_or("anonymous")
                .to_string();

            let exp = claims
                .get("exp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let permissions: HashSet<String> = claims
                .get("permissions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            Ok(AuthSession {
                user,
                token: token.to_string(),
                expires_at: SystemTime::UNIX_EPOCH + Duration::from_secs(exp as u64),
                permissions,
                provider: AuthProvider::Jwt,
                created_at: SystemTime::now(),
            })
        } else {
            Err(AuthError::AuthenticationFailed("no JWT secret configured".to_string()))
        }
    }

    async fn authenticate_basic(&self, token: &str) -> Result<AuthSession, AuthError> {
        // Basic auth token is base64-encoded "username:password"
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(token)
            .map_err(|_| AuthError::InvalidToken("invalid base64".to_string()))?;
        let credentials = String::from_utf8(decoded)
            .map_err(|_| AuthError::InvalidToken("invalid UTF-8".to_string()))?;

        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidToken("invalid basic auth format".to_string()));
        }

        Ok(AuthSession {
            user: parts[0].to_string(),
            token: token.to_string(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            permissions: HashSet::new(),
            provider: AuthProvider::Basic,
            created_at: SystemTime::now(),
        })
    }

    async fn authenticate_oauth2(&self, token: &str) -> Result<AuthSession, AuthError> {
        // OAuth2 token validation would call the provider's introspection endpoint
        // For now, create a basic session
        Ok(AuthSession {
            user: "oauth-user".to_string(),
            token: token.to_string(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            permissions: HashSet::new(),
            provider: AuthProvider::OAuth2,
            created_at: SystemTime::now(),
        })
    }
}

use base64::Engine as _;

fn action_to_permission(action: &str) -> String {
    match action {
        "publish" => "package:publish".to_string(),
        "install" => "package:install".to_string(),
        "unpublish" => "package:unpublish".to_string(),
        "deprecate" => "package:deprecate".to_string(),
        "read" => "package:read".to_string(),
        "search" => "registry:search".to_string(),
        _ => format!("custom:{}", action),
    }
}

/// Helper to create a JWT token for testing or internal use.
pub fn create_jwt_token(
    user: &str,
    secret: &str,
    permissions: &[&str],
    expiry_secs: u64,
) -> Result<String, AuthError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = serde_json::json!({
        "sub": user,
        "exp": now + expiry_secs,
        "iat": now,
        "permissions": permissions,
    });

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
    let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| AuthError::AuthenticationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_authenticate() {
        let store = Arc::new(MemoryTokenStore::new());
        let authenticator = Authenticator::new(
            AuthProvider::Jwt,
            Some("test-secret".to_string()),
            store,
        );

        let token = create_jwt_token("testuser", "test-secret", &["package:read"], 3600).unwrap();
        let session = authenticator.authenticate(&token).await.unwrap();
        assert_eq!(session.user, "testuser");
        assert!(session.has_permission("package:read"));
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_jwt_expired() {
        let store = Arc::new(MemoryTokenStore::new());
        let authenticator = Authenticator::new(
            AuthProvider::Jwt,
            Some("test-secret".to_string()),
            store,
        );

        let token = create_jwt_token("testuser", "test-secret", &[], 0).unwrap();
        // Wait a tiny bit for the token to expire
        tokio::time::sleep(Duration::from_millis(10)).await;
        let session = authenticator.authenticate(&token).await.unwrap();
        assert!(session.is_expired());
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let store = Arc::new(MemoryTokenStore::new());
        let authenticator = Authenticator::new(
            AuthProvider::Jwt,
            Some("test-secret".to_string()),
            store,
        );

        let result = authenticator.authenticate("invalid-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorize() {
        let store = Arc::new(MemoryTokenStore::new());
        let authenticator = Authenticator::new(
            AuthProvider::Jwt,
            Some("test-secret".to_string()),
            store,
        );

        let token = create_jwt_token("testuser", "test-secret", &["package:publish"], 3600).unwrap();
        let session = authenticator.authenticate(&token).await.unwrap();

        let result = authenticator.authorize(&session, "publish").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authorize_denied() {
        let store = Arc::new(MemoryTokenStore::new());
        let authenticator = Authenticator::new(
            AuthProvider::Jwt,
            Some("test-secret".to_string()),
            store,
        );

        let token = create_jwt_token("testuser", "test-secret", &[], 3600).unwrap();
        let session = authenticator.authenticate(&token).await.unwrap();

        let result = authenticator.authorize(&session, "publish").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_provider_display() {
        assert_eq!(AuthProvider::Jwt.to_string(), "jwt");
        assert_eq!(AuthProvider::OAuth2.to_string(), "oauth2");
        assert_eq!(AuthProvider::Basic.to_string(), "basic");
    }

    #[test]
    fn test_session_ttl() {
        let session = AuthSession {
            user: "test".to_string(),
            token: "token".to_string(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            permissions: HashSet::new(),
            provider: AuthProvider::Jwt,
            created_at: SystemTime::now(),
        };
        assert!(session.ttl() > Duration::ZERO);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_memory_token_store() {
        let store = MemoryTokenStore::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            store
                .store_token("user1", "token1", SystemTime::now() + Duration::from_secs(3600))
                .await
                .unwrap();
            let token = store.get_token("user1").await.unwrap();
            assert_eq!(token, Some("token1".to_string()));
            store.remove_token("user1").await.unwrap();
            let token = store.get_token("user1").await.unwrap();
            assert!(token.is_none());
        });
    }
}
