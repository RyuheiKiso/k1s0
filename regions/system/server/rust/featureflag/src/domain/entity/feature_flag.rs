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

/// FeatureFlag はフィーチャーフラグのドメインエンティティ。
/// STATIC-CRITICAL-001 監査対応: tenant_id でテナント分離を実現する。
#[derive(Debug, Clone)]
pub struct FeatureFlag {
    pub id: Uuid,
    /// テナント識別子: テナントごとのフラグ分離に使用する。
    pub tenant_id: Uuid,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
    pub rules: Vec<FlagRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FeatureFlag {
    pub fn new(tenant_id: Uuid, flag_key: String, description: String, enabled: bool) -> Self {
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
