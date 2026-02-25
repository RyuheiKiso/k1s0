use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::secret::{Secret, SecretValue, SecretVersion};
use crate::domain::repository::SecretStore;
use crate::infrastructure::encryption::MasterKey;

/// SecretStorePostgresRepository は PostgreSQL を使った SecretStore の実装。
/// すべてのシークレットデータは MasterKey で暗号化された状態で保存される。
pub struct SecretStorePostgresRepository {
    pool: Arc<PgPool>,
    master_key: Arc<MasterKey>,
}

impl SecretStorePostgresRepository {
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

/// vault.secret_versions テーブルの行を表す構造体。
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

#[async_trait]
impl SecretStore for SecretStorePostgresRepository {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret> {
        // secrets テーブルからメタデータを取得
        let secret_row: SecretRow = sqlx::query_as(
            "SELECT id, key_path, current_version, metadata, created_at, updated_at \
             FROM vault.secrets WHERE key_path = $1",
        )
        .bind(path)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))?;

        // バージョン行を取得（指定バージョンがあればそれだけ、なければ全バージョン）
        let version_rows: Vec<SecretVersionRow> = if let Some(v) = version {
            sqlx::query_as(
                "SELECT id, secret_id, version, encrypted_data, nonce, created_at \
                 FROM vault.secret_versions WHERE secret_id = $1 AND version = $2 \
                 ORDER BY version ASC",
            )
            .bind(secret_row.id)
            .bind(v as i32)
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

        if version.is_some() && version_rows.is_empty() {
            return Err(anyhow::anyhow!(
                "version {} not found for secret: {}",
                version.unwrap(),
                path
            ));
        }

        // 各バージョンを復号化して SecretVersion に変換
        let mut versions = Vec::with_capacity(version_rows.len());
        for row in &version_rows {
            let plaintext = self
                .master_key
                .decrypt(&row.encrypted_data, &row.nonce)?;
            let data: HashMap<String, String> = serde_json::from_slice(&plaintext)?;
            versions.push(SecretVersion {
                version: row.version as i64,
                value: SecretValue { data },
                created_at: row.created_at,
                destroyed: false,
            });
        }

        Ok(Secret {
            path: secret_row.key_path,
            current_version: secret_row.current_version as i64,
            versions,
            created_at: secret_row.created_at,
            updated_at: secret_row.updated_at,
        })
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        // データを JSON にシリアライズして暗号化
        let plaintext = serde_json::to_vec(&data)?;
        let (encrypted_data, nonce) = self.master_key.encrypt(&plaintext)?;

        let mut tx = self.pool.begin().await?;

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
                "UPDATE vault.secrets SET current_version = $2 WHERE id = $1",
            )
            .bind(id)
            .bind(new_ver)
            .execute(&mut *tx)
            .await?;
            (id, new_ver)
        } else {
            let id: (uuid::Uuid,) = sqlx::query_as(
                "INSERT INTO vault.secrets (key_path, current_version) VALUES ($1, 1) RETURNING id",
            )
            .bind(path)
            .fetch_one(&mut *tx)
            .await?;
            (id.0, 1)
        };

        // secret_versions テーブルに暗号化データを挿入
        sqlx::query(
            "INSERT INTO vault.secret_versions (secret_id, version, encrypted_data, nonce) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(secret_id)
        .bind(new_version)
        .bind(&encrypted_data)
        .bind(&nonce)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(new_version as i64)
    }

    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()> {
        let secret_row: Option<(uuid::Uuid,)> =
            sqlx::query_as("SELECT id FROM vault.secrets WHERE key_path = $1")
                .bind(path)
                .fetch_optional(self.pool.as_ref())
                .await?;

        let (secret_id,) = secret_row
            .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))?;

        if versions.is_empty() {
            // 全バージョン削除 → CASCADE で secret_versions も削除される
            sqlx::query("DELETE FROM vault.secrets WHERE id = $1")
                .bind(secret_id)
                .execute(self.pool.as_ref())
                .await?;
        } else {
            // 指定バージョンのみ削除
            let version_ints: Vec<i32> = versions.iter().map(|v| *v as i32).collect();
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
        let like_pattern = format!("{}%", path_prefix);
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT key_path FROM vault.secrets WHERE key_path LIKE $1 ORDER BY key_path",
        )
        .bind(&like_pattern)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(|(p,)| p).collect())
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1::BIGINT FROM vault.secrets WHERE key_path = $1")
                .bind(path)
                .fetch_optional(self.pool.as_ref())
                .await?;

        Ok(row.is_some())
    }
}

#[cfg(test)]
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
    fn test_encrypt_decrypt_data_roundtrip() {
        // MasterKey を使った暗号化/復号化の統合テスト
        let master_key = MasterKey::from_env().unwrap();

        let data = HashMap::from([
            ("password".to_string(), "s3cret".to_string()),
            ("host".to_string(), "localhost".to_string()),
        ]);
        let plaintext = serde_json::to_vec(&data).unwrap();

        let (encrypted, nonce) = master_key.encrypt(&plaintext).unwrap();
        let decrypted = master_key.decrypt(&encrypted, &nonce).unwrap();

        let restored: HashMap<String, String> = serde_json::from_slice(&decrypted).unwrap();
        assert_eq!(restored["password"], "s3cret");
        assert_eq!(restored["host"], "localhost");
    }
}
