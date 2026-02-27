use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ApiKey は API キーを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: String,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CreateApiKeyRequest は API キー作成リクエスト。
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub tenant_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// CreateApiKeyResponse は API キー作成レスポンス。
/// raw_key は作成直後にのみ返される平文キー。
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub raw_key: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// ApiKeySummary はキー一覧表示用の要約。
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&ApiKey> for ApiKeySummary {
    fn from(key: &ApiKey) -> Self {
        Self {
            id: key.id,
            name: key.name.clone(),
            prefix: key.prefix.clone(),
            scopes: key.scopes.clone(),
            expires_at: key.expires_at,
            revoked: key.revoked,
            created_at: key.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_creation() {
        let now = Utc::now();
        let key = ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "My API Key".to_string(),
            key_hash: "hashed_value".to_string(),
            prefix: "k1s0_ab12".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
            expires_at: None,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(key.tenant_id, "tenant-1");
        assert_eq!(key.name, "My API Key");
        assert!(!key.revoked);
        assert_eq!(key.scopes.len(), 2);
    }

    #[test]
    fn test_api_key_summary_from() {
        let now = Utc::now();
        let key = ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Test Key".to_string(),
            key_hash: "hash".to_string(),
            prefix: "k1s0_cd34".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: Some(now + chrono::Duration::days(30)),
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        let summary = ApiKeySummary::from(&key);
        assert_eq!(summary.id, key.id);
        assert_eq!(summary.name, "Test Key");
        assert_eq!(summary.prefix, "k1s0_cd34");
        assert!(!summary.revoked);
    }

    #[test]
    fn test_api_key_serialization_roundtrip() {
        let now = Utc::now();
        let key = ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Roundtrip Key".to_string(),
            key_hash: "hashed".to_string(),
            prefix: "k1s0_ef56".to_string(),
            scopes: vec!["admin".to_string()],
            expires_at: None,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&key).unwrap();
        let deserialized: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(key, deserialized);
    }
}
