use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagRule {
    pub attribute: String,
    pub operator: String, // "eq", "contains", "in"
    pub value: String,
    pub variant: String,
}

/// `FeatureFlag` はフィーチャーフラグのドメインエンティティ。
/// STATIC-CRITICAL-001 監査対応: `tenant_id` でテナント分離を実現する。
/// HIGH-005 対応: migration 006 で `tenant_id` を UUID → TEXT に変更したため String 型を使用する。
#[derive(Debug, Clone)]
pub struct FeatureFlag {
    pub id: Uuid,
    /// テナント識別子: migration 006 で TEXT 型に変更済み。
    pub tenant_id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
    pub rules: Vec<FlagRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FeatureFlag {
    /// HIGH-005 対応: `tenant_id` は String 型（DB の TEXT 型に対応）。
    #[must_use]
    pub fn new(tenant_id: String, flag_key: String, description: String, enabled: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            flag_key,
            description,
            enabled,
            variants: vec![],
            rules: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}
