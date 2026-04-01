use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::feature_flag::{FeatureFlag, FlagRule, FlagVariant};
use crate::domain::repository::FeatureFlagRepository;

/// FeatureFlagPostgresRepository は PostgreSQL を使ったフィーチャーフラグリポジトリ。
pub struct FeatureFlagPostgresRepository {
    pool: Arc<PgPool>,
}

impl FeatureFlagPostgresRepository {
    /// 新しい FeatureFlagPostgresRepository を作成する。
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// PostgreSQL の行をマッピングするための内部構造体。
/// STATIC-CRITICAL-001 監査対応: tenant_id カラムを含む。
#[derive(sqlx::FromRow)]
struct FeatureFlagRow {
    id: Uuid,
    tenant_id: Uuid,
    flag_key: String,
    description: String,
    enabled: bool,
    variants: serde_json::Value,
    rules: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<FeatureFlagRow> for FeatureFlag {
    fn from(row: FeatureFlagRow) -> Self {
        let variants: Vec<FlagVariant> = serde_json::from_value(row.variants).unwrap_or_default();
        let rules: Vec<FlagRule> = serde_json::from_value(row.rules).unwrap_or_default();
        FeatureFlag {
            id: row.id,
            tenant_id: row.tenant_id,
            flag_key: row.flag_key,
            description: row.description,
            enabled: row.enabled,
            variants,
            rules,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl FeatureFlagRepository for FeatureFlagPostgresRepository {
    /// STATIC-CRITICAL-001 監査対応: tenant_id + flag_key でフラグを取得する。
    async fn find_by_key(&self, tenant_id: Uuid, flag_key: &str) -> anyhow::Result<FeatureFlag> {
        let row: Option<FeatureFlagRow> = sqlx::query_as(
            "SELECT id, tenant_id, flag_key, description, enabled, variants, rules, created_at, updated_at \
             FROM featureflag.feature_flags WHERE tenant_id = $1 AND flag_key = $2",
        )
        .bind(tenant_id)
        .bind(flag_key)
        .fetch_optional(self.pool.as_ref())
        .await?;

        row.map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("flag not found: {}", flag_key))
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内の全フラグを取得する。
    async fn find_all(&self, tenant_id: Uuid) -> anyhow::Result<Vec<FeatureFlag>> {
        let rows: Vec<FeatureFlagRow> = sqlx::query_as(
            "SELECT id, tenant_id, flag_key, description, enabled, variants, rules, created_at, updated_at \
             FROM featureflag.feature_flags WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフラグを作成する。
    async fn create(&self, tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()> {
        let variants_json = serde_json::to_value(&flag.variants)?;
        let rules_json = serde_json::to_value(&flag.rules)?;

        sqlx::query(
            "INSERT INTO featureflag.feature_flags \
             (id, tenant_id, flag_key, description, enabled, variants, rules, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(flag.id)
        .bind(tenant_id)
        .bind(&flag.flag_key)
        .bind(&flag.description)
        .bind(flag.enabled)
        .bind(&variants_json)
        .bind(&rules_json)
        .bind(flag.created_at)
        .bind(flag.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内のフラグを更新する。
    async fn update(&self, tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()> {
        let variants_json = serde_json::to_value(&flag.variants)?;
        let rules_json = serde_json::to_value(&flag.rules)?;

        let result = sqlx::query(
            "UPDATE featureflag.feature_flags \
             SET description = $3, enabled = $4, variants = $5, rules = $6, updated_at = $7 \
             WHERE tenant_id = $1 AND flag_key = $2",
        )
        .bind(tenant_id)
        .bind(&flag.flag_key)
        .bind(&flag.description)
        .bind(flag.enabled)
        .bind(&variants_json)
        .bind(&rules_json)
        .bind(flag.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("flag not found: {}", flag.flag_key));
        }

        Ok(())
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内のフラグを削除する。
    async fn delete(&self, tenant_id: Uuid, id: &Uuid) -> anyhow::Result<bool> {
        let result =
            sqlx::query("DELETE FROM featureflag.feature_flags WHERE tenant_id = $1 AND id = $2")
                .bind(tenant_id)
                .bind(id)
                .execute(self.pool.as_ref())
                .await?;

        Ok(result.rows_affected() > 0)
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内でのフラグキー存在確認。
    async fn exists_by_key(&self, tenant_id: Uuid, flag_key: &str) -> anyhow::Result<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM featureflag.feature_flags WHERE tenant_id = $1 AND flag_key = $2",
        )
        .bind(tenant_id)
        .bind(flag_key)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(count.0 > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// システムテナントUUID: テスト共通
    fn system_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[test]
    fn test_feature_flag_row_conversion() {
        let row = FeatureFlagRow {
            id: Uuid::new_v4(),
            tenant_id: system_tenant(),
            flag_key: "test-flag".to_string(),
            description: "A test flag".to_string(),
            enabled: true,
            variants: serde_json::json!([
                {"name": "control", "value": "off", "weight": 50},
                {"name": "treatment", "value": "on", "weight": 50}
            ]),
            rules: serde_json::json!([
                {"attribute": "tenant_id", "operator": "eq", "value": "acme", "variant": "treatment"}
            ]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let flag: FeatureFlag = row.into();
        assert_eq!(flag.flag_key, "test-flag");
        assert_eq!(flag.description, "A test flag");
        assert!(flag.enabled);
        assert_eq!(flag.variants.len(), 2);
        assert_eq!(flag.variants[0].name, "control");
        assert_eq!(flag.variants[1].name, "treatment");
        assert_eq!(flag.rules.len(), 1);
        assert_eq!(flag.rules[0].attribute, "tenant_id");
    }

    #[test]
    fn test_feature_flag_row_conversion_empty_json() {
        let row = FeatureFlagRow {
            id: Uuid::new_v4(),
            tenant_id: system_tenant(),
            flag_key: "empty-flag".to_string(),
            description: "".to_string(),
            enabled: false,
            variants: serde_json::json!([]),
            rules: serde_json::json!([]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let flag: FeatureFlag = row.into();
        assert_eq!(flag.flag_key, "empty-flag");
        assert!(!flag.enabled);
        assert!(flag.variants.is_empty());
        assert!(flag.rules.is_empty());
    }

    #[test]
    fn test_feature_flag_row_conversion_invalid_json_fallback() {
        let row = FeatureFlagRow {
            id: Uuid::new_v4(),
            tenant_id: system_tenant(),
            flag_key: "invalid-json-flag".to_string(),
            description: "".to_string(),
            enabled: false,
            variants: serde_json::json!("not an array"),
            rules: serde_json::json!(null),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let flag: FeatureFlag = row.into();
        // unwrap_or_default により空 Vec になる
        assert!(flag.variants.is_empty());
        assert!(flag.rules.is_empty());
    }
}
