use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
// HIGH-003 監査対応: LIKE 検索前に %_\ をエスケープして意図しない全件マッチを防止する
use k1s0_server_common::escape_like_pattern;
use sqlx::PgPool;

use crate::domain::entity::secret::{Secret, SecretValue, SecretVersion};
use crate::domain::repository::SecretStore;
use crate::infrastructure::encryption::MasterKey;

/// `SecretStorePostgresRepository` は `PostgreSQL` を使った `SecretStore` の実装。
/// すべてのシークレットデータは `MasterKey` で暗号化された状態で保存される。
pub struct SecretStorePostgresRepository {
    pool: Arc<PgPool>,
    master_key: Arc<MasterKey>,
}

impl SecretStorePostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>, master_key: Arc<MasterKey>) -> Self {
        Self { pool, master_key }
    }
}

/// vault.secrets テーブルの行を表す構造体。
#[derive(sqlx::FromRow)]
struct SecretRow {
    id: uuid::Uuid,
    key_path: String,
    current_version: i32,
    #[allow(dead_code)]
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// `vault.secret_versions` テーブルの行を表す構造体。
#[derive(sqlx::FromRow)]
struct SecretVersionRow {
    #[allow(dead_code)]
    id: uuid::Uuid,
    #[allow(dead_code)]
    secret_id: uuid::Uuid,
    version: i32,
    encrypted_data: Vec<u8>,
    nonce: Vec<u8>,
    created_at: DateTime<Utc>,
}

/// ADR-0109 対応: `key_path` の先頭セグメントからテナント ID を抽出する。
/// `key_path` は "{`tenant_id`}/..." の形式であること。
/// 先頭セグメントが取れない場合はフォールバックとして "system" を返す。
fn extract_tenant_id(path: &str) -> &str {
    path.split('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("system")
}

#[async_trait]
impl SecretStore for SecretStorePostgresRepository {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret> {
        let tenant_id = extract_tenant_id(path);

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault.secrets は RLS FORCE が有効（migration 007）。
        // クエリ前にセッション変数を設定してテナント分離を有効にする。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        // secrets テーブルからメタデータを取得
        let secret_row: SecretRow = sqlx::query_as(
            "SELECT id, key_path, current_version, metadata, created_at, updated_at \
             FROM vault.secrets WHERE key_path = $1",
        )
        .bind(path)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| anyhow::anyhow!("secret not found: {path}"))?;

        // バージョン行を取得（指定バージョンがあればそれだけ、なければ全バージョン）
        let version_rows: Vec<SecretVersionRow> = if let Some(v) = version {
            sqlx::query_as(
                "SELECT id, secret_id, version, encrypted_data, nonce, created_at \
                 FROM vault.secret_versions WHERE secret_id = $1 AND version = $2 \
                 ORDER BY version ASC",
            )
            .bind(secret_row.id)
            // LOW-008: 安全な型変換（オーバーフロー防止）
            .bind(i32::try_from(v).unwrap_or(i32::MAX))
            .fetch_all(self.pool.as_ref())
            .await?
        } else {
            sqlx::query_as(
                "SELECT id, secret_id, version, encrypted_data, nonce, created_at \
                 FROM vault.secret_versions WHERE secret_id = $1 \
                 ORDER BY version ASC",
            )
            .bind(secret_row.id)
            .fetch_all(self.pool.as_ref())
            .await?
        };

        if let Some(v) = version {
            if version_rows.is_empty() {
                return Err(anyhow::anyhow!("version {v} not found for secret: {path}"));
            }
        }

        // CRIT-003 監査対応: シークレットのパスを AAD として渡し、ciphertext swap attack を防止する
        // HIGH-RUST-001 Phase B 完了: decrypt_with_legacy_fallback を削除し decrypt に統一した
        let mut versions = Vec::with_capacity(version_rows.len());
        for row in &version_rows {
            let plaintext =
                self.master_key
                    .decrypt(&row.encrypted_data, &row.nonce, path.as_bytes())?;
            let data: HashMap<String, String> = serde_json::from_slice(&plaintext)?;
            versions.push(SecretVersion {
                version: i64::from(row.version),
                value: SecretValue { data },
                created_at: row.created_at,
                destroyed: false,
            });
        }

        Ok(Secret {
            path: secret_row.key_path,
            current_version: i64::from(secret_row.current_version),
            versions,
            created_at: secret_row.created_at,
            updated_at: secret_row.updated_at,
        })
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        let tenant_id = extract_tenant_id(path);

        // データを JSON にシリアライズして暗号化
        // CRIT-003 監査対応: シークレットのパスを AAD として渡し、ciphertext swap attack を防止する
        let plaintext = serde_json::to_vec(&data)?;
        let (encrypted_data, nonce) = self.master_key.encrypt(&plaintext, path.as_bytes())?;

        let mut tx = self.pool.begin().await?;

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault テーブルは RLS FORCE が有効（migration 007）。
        // トランザクション内でセッション変数を設定して全操作にテナント分離を適用する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // secrets テーブルに UPSERT
        let row: Option<(uuid::Uuid, i32)> = sqlx::query_as(
            "SELECT id, current_version FROM vault.secrets WHERE key_path = $1 FOR UPDATE",
        )
        .bind(path)
        .fetch_optional(&mut *tx)
        .await?;

        let (secret_id, new_version) = if let Some((id, current)) = row {
            let new_ver = current + 1;
            sqlx::query(
                "UPDATE vault.secrets SET current_version = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(id)
            .bind(new_ver)
            .execute(&mut *tx)
            .await?;
            (id, new_ver)
        } else {
            // migration 007 で tenant_id NOT NULL DEFAULT 削除済み: INSERT に明示指定が必要
            let id: (uuid::Uuid,) = sqlx::query_as(
                "INSERT INTO vault.secrets (tenant_id, key_path, current_version) \
                 VALUES ($1, $2, 1) RETURNING id",
            )
            .bind(tenant_id)
            .bind(path)
            .fetch_one(&mut *tx)
            .await?;
            (id.0, 1)
        };

        // migration 007 で tenant_id NOT NULL DEFAULT 削除済み: INSERT に明示指定が必要
        sqlx::query(
            "INSERT INTO vault.secret_versions \
             (tenant_id, secret_id, version, encrypted_data, nonce) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(tenant_id)
        .bind(secret_id)
        .bind(new_version)
        .bind(&encrypted_data)
        .bind(&nonce)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(i64::from(new_version))
    }

    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()> {
        let tenant_id = extract_tenant_id(path);

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault テーブルは RLS FORCE が有効（migration 007）。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        let secret_row: Option<(uuid::Uuid,)> =
            sqlx::query_as("SELECT id FROM vault.secrets WHERE key_path = $1")
                .bind(path)
                .fetch_optional(self.pool.as_ref())
                .await?;

        let (secret_id,) = secret_row.ok_or_else(|| anyhow::anyhow!("secret not found: {path}"))?;

        if versions.is_empty() {
            // 全バージョン削除 → CASCADE で secret_versions も削除される
            sqlx::query("DELETE FROM vault.secrets WHERE id = $1")
                .bind(secret_id)
                .execute(self.pool.as_ref())
                .await?;
        } else {
            // 指定バージョンのみ削除
            // LOW-008: 安全な型変換（オーバーフロー防止）
            let version_ints: Vec<i32> = versions.iter().map(|v| i32::try_from(*v).unwrap_or(i32::MAX)).collect();
            sqlx::query(
                "DELETE FROM vault.secret_versions \
                 WHERE secret_id = $1 AND version = ANY($2)",
            )
            .bind(secret_id)
            .bind(&version_ints)
            .execute(self.pool.as_ref())
            .await?;
        }

        Ok(())
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        let tenant_id = extract_tenant_id(path_prefix);

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault テーブルは RLS FORCE が有効（migration 007）。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        // HIGH-003 監査対応: LIKE 検索のワイルドカード特殊文字をエスケープし、ESCAPE '\' を指定する
        let like_pattern = format!("{}%", escape_like_pattern(path_prefix));
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT key_path FROM vault.secrets \
             WHERE key_path LIKE $1 ESCAPE '\\' ORDER BY key_path",
        )
        .bind(&like_pattern)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(|(p,)| p).collect())
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        let tenant_id = extract_tenant_id(path);

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault テーブルは RLS FORCE が有効（migration 007）。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1::BIGINT FROM vault.secrets WHERE key_path = $1")
                .bind(path)
                .fetch_optional(self.pool.as_ref())
                .await?;

        Ok(row.is_some())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_row_mapping() {
        // SecretRow 構造体のフィールドが正しい型であることの静的検証
        let _row = SecretRow {
            id: uuid::Uuid::new_v4(),
            key_path: "test/path".to_string(),
            current_version: 1,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(_row.key_path, "test/path");
        assert_eq!(_row.current_version, 1);
    }

    #[test]
    fn test_secret_version_row_mapping() {
        let _row = SecretVersionRow {
            id: uuid::Uuid::new_v4(),
            secret_id: uuid::Uuid::new_v4(),
            version: 1,
            encrypted_data: vec![0u8; 32],
            nonce: vec![0u8; 12],
            created_at: Utc::now(),
        };
        assert_eq!(_row.version, 1);
        assert_eq!(_row.nonce.len(), 12);
    }

    #[test]
    fn test_extract_tenant_id() {
        // ADR-0109 パス形式 "{tenant_id}/..." からの抽出を検証する
        assert_eq!(extract_tenant_id("tenant-1/app/db"), "tenant-1");
        assert_eq!(extract_tenant_id("system/config/key"), "system");
        assert_eq!(extract_tenant_id("single"), "single");
        // 空パスのフォールバックを検証する
        assert_eq!(extract_tenant_id(""), "system");
    }

    #[test]
    fn test_encrypt_decrypt_data_roundtrip() {
        // CRIT-003 監査対応: MasterKey を使った暗号化/復号化の統合テスト
        // シークレットパスを AAD として渡すことで ciphertext swap attack への耐性を確認する
        let master_key = MasterKey::from_env().unwrap();
        let path = "test/my-service/db-password";

        let data = HashMap::from([
            ("password".to_string(), "s3cret".to_string()),
            ("host".to_string(), "localhost".to_string()),
        ]);
        let plaintext = serde_json::to_vec(&data).unwrap();

        let (encrypted, nonce) = master_key.encrypt(&plaintext, path.as_bytes()).unwrap();
        let decrypted = master_key
            .decrypt(&encrypted, &nonce, path.as_bytes())
            .unwrap();

        let restored: HashMap<String, String> = serde_json::from_slice(&decrypted).unwrap();
        assert_eq!(restored["password"], "s3cret");
        assert_eq!(restored["host"], "localhost");
    }
}
