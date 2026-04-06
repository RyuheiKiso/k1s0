use std::sync::Arc;

use async_trait::async_trait;
// M-02 監査対応: LIKE/ILIKE ワイルドカードエスケープのため server-common のユーティリティを使用する
use k1s0_server_common::escape_like_pattern;
use sqlx::{PgPool, Row};
use uuid::Uuid;
#[cfg(feature = "db-tests")]
use uuid::Uuid as _Uuid;

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use crate::domain::error::ConfigRepositoryError;
use crate::domain::repository::ConfigRepository;

/// ConfigPostgresRepository は ConfigRepository の PostgreSQL 実装。
/// STATIC-HIGH-002: AES-256-GCM 暗号化による機密設定値の保護をサポートする。
pub struct ConfigPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
    /// AES-256 暗号化鍵。None の場合は暗号化無効（スタブ/開発環境）
    encryption_key: Option<[u8; 32]>,
    /// 暗号化対象の namespace プレフィックスリスト（例: "system.auth", "system.database"）
    sensitive_namespaces: Vec<String>,
}

impl ConfigPostgresRepository {
    #[cfg(feature = "db-tests")]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
            encryption_key: None,
            sensitive_namespaces: vec![],
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
            encryption_key: None,
            sensitive_namespaces: vec![],
        }
    }

    /// STATIC-HIGH-002: 暗号化鍵と機密 namespace リストを設定するビルダーメソッド。
    pub fn set_encryption(mut self, key: [u8; 32], sensitive_namespaces: Vec<String>) -> Self {
        self.encryption_key = Some(key);
        self.sensitive_namespaces = sensitive_namespaces;
        self
    }

    /// テストやマイグレーション用にプールへの参照を返す。
    #[cfg(feature = "db-tests")]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 指定した namespace が暗号化対象かどうかを判定する。
    /// 暗号化鍵が設定されており、かつ sensitive_namespaces のいずれかに前方一致する場合に true を返す。
    fn is_sensitive_namespace(&self, namespace: &str) -> bool {
        self.encryption_key.is_some()
            && self
                .sensitive_namespaces
                .iter()
                .any(|prefix| namespace.starts_with(prefix.as_str()))
    }

    /// value_json を AES-256-GCM で暗号化し、(value_json_to_store, encrypted_value, is_encrypted) を返す。
    /// 機密 namespace の場合: value_json = '{}'（空 JSON）、encrypted_value = 暗号文（base64）、is_encrypted = true
    /// 非機密 namespace の場合: value_json = 元の値、encrypted_value = NULL、is_encrypted = false
    /// C-001 監査対応: AAD に namespace バイト列を使用し、ciphertext swap attack を防止する。
    fn encrypt_value(
        &self,
        namespace: &str,
        value_json: &serde_json::Value,
    ) -> Result<(serde_json::Value, Option<String>, bool), ConfigRepositoryError> {
        if self.is_sensitive_namespace(namespace) {
            let key = self.encryption_key.as_ref().unwrap(); // is_sensitive_namespace で Some を確認済み
            let plaintext = value_json.to_string();
            // C-001 監査対応: AAD に namespace を指定し、namespace をまたいだ ciphertext の流用を防ぐ
            let ciphertext = k1s0_encryption::aes_encrypt(key, plaintext.as_bytes(), namespace.as_bytes())
                .map_err(|e| {
                    ConfigRepositoryError::Infrastructure(anyhow::anyhow!(
                        "設定値の暗号化に失敗: {}",
                        e
                    ))
                })?;
            // 暗号化済みエントリの value_json は空 JSON を格納し、実値は encrypted_value に保持する
            Ok((
                serde_json::Value::Object(Default::default()),
                Some(ciphertext),
                true,
            ))
        } else {
            Ok((value_json.clone(), None, false))
        }
    }

    #[cfg(feature = "db-tests")]
    pub async fn create(
        &self,
        tenant_id: Uuid,
        entry: &ConfigEntry,
    ) -> anyhow::Result<ConfigEntry> {
        let start = std::time::Instant::now();

        // STATIC-HIGH-002: 機密 namespace の場合は暗号化する
        let (value_json_to_store, encrypted_value, is_encrypted) = self
            .encrypt_value(&entry.namespace, &entry.value_json)
            .map_err(|e| anyhow::anyhow!("暗号化エラー: {}", e))?;

        let row = sqlx::query(
            r#"
            INSERT INTO config_entries (id, tenant_id, namespace, key, value_json, encrypted_value, is_encrypted, version, description, created_by, updated_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, namespace, key, value_json, encrypted_value, is_encrypted, version, description, created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(entry.id)
        // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(tenant_id.to_string())
        .bind(&entry.namespace)
        .bind(&entry.key)
        .bind(&value_json_to_store)
        .bind(&encrypted_value)
        .bind(is_encrypted)
        .bind(entry.version)
        .bind(&entry.description)
        .bind(&entry.created_by)
        .bind(&entry.updated_by)
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .fetch_one(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "config_entries", start.elapsed().as_secs_f64());
        }

        Ok(row_to_config_entry(row, self.encryption_key.as_ref())?)
    }

    #[cfg(feature = "db-tests")]
    pub async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>> {
        let start = std::time::Instant::now();
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, config_entry_id, namespace, key, old_value_json, new_value_json,
                   old_version, new_version, change_type, changed_by, trace_id, created_at
            FROM config_change_logs
            WHERE namespace = $1 AND key = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(namespace)
        .bind(key)
        .fetch_all(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_change_logs",
                "config_change_logs",
                start.elapsed().as_secs_f64(),
            );
        }

        let logs = rows
            .into_iter()
            .map(|row| {
                Ok(ConfigChangeLog {
                    id: row.try_get("id")?,
                    tenant_id: row.try_get("tenant_id")?,
                    config_entry_id: row.try_get("config_entry_id")?,
                    namespace: row.try_get("namespace")?,
                    key: row.try_get("key")?,
                    old_value: row.try_get("old_value_json")?,
                    new_value: row.try_get("new_value_json")?,
                    old_version: row.try_get("old_version")?,
                    new_version: row.try_get("new_version")?,
                    change_type: row.try_get("change_type")?,
                    changed_by: row.try_get("changed_by")?,
                    trace_id: row.try_get("trace_id")?,
                    changed_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(logs)
    }

    #[cfg(feature = "db-tests")]
    pub async fn find_by_id(&self, id: &_Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        let start = std::time::Instant::now();
        let row = sqlx::query(
            r#"
            SELECT id, namespace, key, value_json, encrypted_value, is_encrypted, version, description,
                   created_by, updated_by, created_at, updated_at
            FROM config_entries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_id",
                "config_entries",
                start.elapsed().as_secs_f64(),
            );
        }

        match row {
            Some(row) => Ok(Some(row_to_config_entry(row, self.encryption_key.as_ref())?)),
            None => Ok(None),
        }
    }
}

/// PostgreSQL の行から ConfigEntry を構築するヘルパー。
/// STATIC-HIGH-002: is_encrypted = true の場合、encrypted_value を復号して value_json として返す。
/// C-001 監査対応: 復号時の AAD に namespace を使用し、暗号化時と同一コンテキストを検証する。
fn row_to_config_entry(
    row: sqlx::postgres::PgRow,
    encryption_key: Option<&[u8; 32]>,
) -> Result<ConfigEntry, anyhow::Error> {
    let description: Option<String> = row.try_get("description")?;
    let is_encrypted: bool = row.try_get("is_encrypted").unwrap_or(false);
    let encrypted_value: Option<String> = row.try_get("encrypted_value").unwrap_or(None);
    // C-001 監査対応: AAD 検証のため namespace を先に取得する
    let namespace: String = row.try_get("namespace")?;

    // 暗号化済みエントリの場合は復号して value_json として使用する
    let value_json = if is_encrypted {
        match (encryption_key, encrypted_value.as_deref()) {
            (Some(key), Some(ciphertext)) => {
                // ADR-0104: Phase B（バッチ再暗号化）完了後、aes_decrypt のみで復号する
                let plaintext = k1s0_encryption::aes_decrypt(key, ciphertext, namespace.as_bytes()).map_err(|e| {
                    anyhow::anyhow!("設定値の復号に失敗: {}", e)
                })?;
                serde_json::from_slice(&plaintext).map_err(|e| {
                    anyhow::anyhow!("復号後の JSON パースに失敗: {}", e)
                })?
            }
            // 暗号化フラグが true だが鍵がない場合は空 JSON を返す（設定ミス対策）
            _ => serde_json::Value::Object(Default::default()),
        }
    } else {
        row.try_get("value_json")?
    };

    Ok(ConfigEntry {
        id: row.try_get("id")?,
        namespace,
        key: row.try_get("key")?,
        value_json,
        version: row.try_get("version")?,
        description: description.unwrap_or_default(),
        created_by: row.try_get("created_by")?,
        updated_by: row.try_get("updated_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// PostgreSQL の行から ServiceConfigEntry を構築するヘルパー。
/// STATIC-HIGH-002: is_encrypted = true の場合、encrypted_value を復号して value として返す。
/// C-001 監査対応: 復号時の AAD に namespace を使用し、暗号化時と同一コンテキストを検証する。
fn row_to_service_config_entry(
    row: sqlx::postgres::PgRow,
    encryption_key: Option<&[u8; 32]>,
) -> Result<ServiceConfigEntry, anyhow::Error> {
    let is_encrypted: bool = row.try_get("is_encrypted").unwrap_or(false);
    let encrypted_value: Option<String> = row.try_get("encrypted_value").unwrap_or(None);
    // C-001 監査対応: AAD 検証のため namespace を先に取得する
    let namespace: String = row.try_get("namespace")?;

    let value = if is_encrypted {
        match (encryption_key, encrypted_value.as_deref()) {
            (Some(key), Some(ciphertext)) => {
                // ADR-0104: Phase B（バッチ再暗号化）完了後、aes_decrypt のみで復号する
                let plaintext = k1s0_encryption::aes_decrypt(key, ciphertext, namespace.as_bytes()).map_err(|e| {
                    anyhow::anyhow!("サービス設定値の復号に失敗: {}", e)
                })?;
                serde_json::from_slice(&plaintext).map_err(|e| {
                    anyhow::anyhow!("復号後の JSON パースに失敗: {}", e)
                })?
            }
            _ => serde_json::Value::Object(Default::default()),
        }
    } else {
        row.try_get("value_json")?
    };

    Ok(ServiceConfigEntry {
        namespace,
        key: row.try_get("key")?,
        value,
        version: row.try_get("version")?,
    })
}

#[async_trait]
impl ConfigRepository for ConfigPostgresRepository {
    /// STATIC-CRITICAL-001 監査対応: tenant_id + namespace + key で設定値を取得する。
    /// STATIC-HIGH-002: is_encrypted = true の場合は復号して返す。
    async fn find_by_namespace_and_key(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<Option<ConfigEntry>, ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        // set_config の第3引数 true は SET LOCAL（トランザクションスコープのみ有効）を意味する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        let start = std::time::Instant::now();
        let row = sqlx::query(
            r#"
            SELECT id, namespace, key, value_json, encrypted_value, is_encrypted, version, description,
                   created_by, updated_by, created_at, updated_at
            FROM config_entries
            WHERE tenant_id = $1 AND namespace = $2 AND key = $3
            "#,
        )
        // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(tenant_id.to_string())
        .bind(namespace)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_namespace_and_key",
                "config_entries",
                start.elapsed().as_secs_f64(),
            );
        }

        match row {
            Some(row) => Ok(Some(
                row_to_config_entry(row, self.encryption_key.as_ref())
                    .map_err(ConfigRepositoryError::Infrastructure)?,
            )),
            None => Ok(None),
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内の namespace 設定値一覧を取得する。
    /// STATIC-HIGH-002: is_encrypted = true のエントリは復号して返す。
    async fn list_by_namespace(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> Result<ConfigListResult, ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        let offset = (page - 1) * page_size;
        let encryption_key = self.encryption_key.as_ref();

        let (entries, total_count) = if let Some(ref search_term) = search {
            // M-02 監査対応: ワイルドカード特殊文字（\, %, _）をエスケープし意図しない全件マッチを防ぐ
            let pattern = format!("%{}%", escape_like_pattern(search_term));
            let start = std::time::Instant::now();
            let rows = sqlx::query(
                r#"
                SELECT id, namespace, key, value_json, encrypted_value, is_encrypted, version, description,
                       created_by, updated_by, created_at, updated_at
                FROM config_entries
                WHERE tenant_id = $1 AND namespace = $2 AND key LIKE $3 ESCAPE '\'
                ORDER BY key ASC
                LIMIT $4 OFFSET $5
                "#,
            )
            // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
            .bind(tenant_id.to_string())
            .bind(namespace)
            .bind(&pattern)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
            if let Some(ref m) = self.metrics {
                m.record_db_query_duration(
                    "list_by_namespace",
                    "config_entries",
                    start.elapsed().as_secs_f64(),
                );
            }

            let entries: Vec<ConfigEntry> = rows
                .into_iter()
                .map(|row| row_to_config_entry(row, encryption_key))
                .collect::<Result<Vec<_>, _>>()
                .map_err(ConfigRepositoryError::Infrastructure)?;

            // HIGH-003 監査対応: COUNT クエリにも ESCAPE '\' を追加し SELECT クエリと整合させる
            let start = std::time::Instant::now();
            let count_row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM config_entries WHERE tenant_id = $1 AND namespace = $2 AND key LIKE $3 ESCAPE '\\'",
            )
            // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
            .bind(tenant_id.to_string())
            .bind(namespace)
            .bind(&pattern)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
            if let Some(ref m) = self.metrics {
                m.record_db_query_duration(
                    "list_by_namespace_count",
                    "config_entries",
                    start.elapsed().as_secs_f64(),
                );
            }

            (entries, count_row.0)
        } else {
            let start = std::time::Instant::now();
            let rows = sqlx::query(
                r#"
                SELECT id, namespace, key, value_json, encrypted_value, is_encrypted, version, description,
                       created_by, updated_by, created_at, updated_at
                FROM config_entries
                WHERE tenant_id = $1 AND namespace = $2
                ORDER BY key ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
            .bind(tenant_id.to_string())
            .bind(namespace)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
            if let Some(ref m) = self.metrics {
                m.record_db_query_duration(
                    "list_by_namespace",
                    "config_entries",
                    start.elapsed().as_secs_f64(),
                );
            }

            let entries: Vec<ConfigEntry> = rows
                .into_iter()
                .map(|row| row_to_config_entry(row, encryption_key))
                .collect::<Result<Vec<_>, _>>()
                .map_err(ConfigRepositoryError::Infrastructure)?;

            let start = std::time::Instant::now();
            let count_row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM config_entries WHERE tenant_id = $1 AND namespace = $2",
            )
            // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
            .bind(tenant_id.to_string())
            .bind(namespace)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
            if let Some(ref m) = self.metrics {
                m.record_db_query_duration(
                    "list_by_namespace_count",
                    "config_entries",
                    start.elapsed().as_secs_f64(),
                );
            }

            (entries, count_row.0)
        };

        let has_next = (i64::from(offset) + i64::from(page_size)) < total_count;

        Ok(ConfigListResult {
            entries,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内の設定値を更新する（楽観的排他制御付き）。
    /// STATIC-HIGH-002: 機密 namespace の場合は AES-256-GCM で暗号化して保存する。
    async fn update(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> Result<ConfigEntry, ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        let now = chrono::Utc::now();
        let new_version = expected_version + 1;

        // STATIC-HIGH-002: 機密 namespace の場合は暗号化する
        let (value_json_to_store, encrypted_value, is_encrypted) =
            self.encrypt_value(namespace, value_json)?;

        let start = std::time::Instant::now();
        let row = sqlx::query(
            r#"
            UPDATE config_entries
            SET value_json = $1,
                encrypted_value = $2,
                is_encrypted = $3,
                version = $4,
                description = COALESCE($5, description),
                updated_by = $6,
                updated_at = $7
            WHERE tenant_id = $8 AND namespace = $9 AND key = $10 AND version = $11
            RETURNING id, namespace, key, value_json, encrypted_value, is_encrypted, version, description, created_by, updated_by, created_at, updated_at
            "#,
        )
        .bind(&value_json_to_store)
        .bind(&encrypted_value)
        .bind(is_encrypted)
        .bind(new_version)
        .bind(&description)
        .bind(updated_by)
        .bind(now)
        // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(tenant_id.to_string())
        .bind(namespace)
        .bind(key)
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("update", "config_entries", start.elapsed().as_secs_f64());
        }

        match row {
            Some(row) => row_to_config_entry(row, self.encryption_key.as_ref())
                .map_err(ConfigRepositoryError::Infrastructure),
            None => {
                // バージョン不一致か、キーが存在しないかを確認
                let exists: Option<(i32,)> = sqlx::query_as(
                    "SELECT version FROM config_entries WHERE tenant_id = $1 AND namespace = $2 AND key = $3",
                )
                // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
                .bind(tenant_id.to_string())
                .bind(namespace)
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

                match exists {
                    // バージョン不一致: 楽観的排他制御エラー
                    Some((current_version,)) => Err(ConfigRepositoryError::VersionConflict {
                        expected: expected_version,
                        current: current_version,
                    }),
                    // キーが存在しない: NotFound エラー
                    None => Err(ConfigRepositoryError::NotFound {
                        namespace: namespace.to_string(),
                        key: key.to_string(),
                    }),
                }
            }
        }
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内の設定値を削除する。
    async fn delete(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<bool, ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        let start = std::time::Instant::now();
        let result = sqlx::query(
            "DELETE FROM config_entries WHERE tenant_id = $1 AND namespace = $2 AND key = $3",
        )
        // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(tenant_id.to_string())
        .bind(namespace)
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "config_entries", start.elapsed().as_secs_f64());
        }

        Ok(result.rows_affected() > 0)
    }

    /// STATIC-CRITICAL-001 監査対応: テナント内のサービス名に紐づく設定値を一括取得する。
    /// STATIC-HIGH-002: is_encrypted = true のエントリは復号して返す。
    async fn find_by_service_name(
        &self,
        tenant_id: Uuid,
        service_name: &str,
    ) -> Result<ServiceConfigResult, ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        // サービス名に紐づく namespace パターンで検索
        // 例: "auth-server" → "system.auth.%" のような namespace にマッチ
        // M-02 監査対応: ハイフンをドットに置換した後に残る特殊文字（\, %, _）をエスケープする
        let escaped_name = escape_like_pattern(&service_name.replace('-', "."));
        let pattern = format!("%.{}%", escaped_name);
        let encryption_key = self.encryption_key.as_ref();

        let start = std::time::Instant::now();
        let rows = sqlx::query(
            r#"
            SELECT namespace, key, value_json, encrypted_value, is_encrypted, version
            FROM config_entries
            WHERE tenant_id = $1 AND namespace LIKE $2 ESCAPE '\'
            ORDER BY namespace, key
            "#,
        )
        // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(tenant_id.to_string())
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_service_name",
                "config_entries",
                start.elapsed().as_secs_f64(),
            );
        }

        let entries: Vec<ServiceConfigEntry> = rows
            .into_iter()
            .map(|row| row_to_service_config_entry(row, encryption_key))
            .collect::<Result<Vec<_>, _>>()
            .map_err(ConfigRepositoryError::Infrastructure)?;

        if entries.is_empty() {
            // service_config_mappings テーブルから直接マッピングを検索
            let mapped_rows = sqlx::query(
                r#"
                SELECT ce.namespace, ce.key, ce.value_json, ce.encrypted_value, ce.is_encrypted, ce.version
                FROM config_entries ce
                INNER JOIN service_config_mappings scm ON ce.id = scm.config_entry_id
                WHERE ce.tenant_id = $1 AND scm.service_name = $2
                ORDER BY ce.namespace, ce.key
                "#,
            )
            // ADR-0093 対応: tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
            .bind(tenant_id.to_string())
            .bind(service_name)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

            let mapped_entries: Vec<ServiceConfigEntry> = mapped_rows
                .into_iter()
                .map(|row| row_to_service_config_entry(row, encryption_key))
                .collect::<Result<Vec<_>, _>>()
                .map_err(ConfigRepositoryError::Infrastructure)?;

            if mapped_entries.is_empty() {
                return Err(ConfigRepositoryError::ServiceNotFound(
                    service_name.to_string(),
                ));
            }

            return Ok(ServiceConfigResult {
                service_name: service_name.to_string(),
                entries: mapped_entries,
            });
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries,
        })
    }

    /// 設定変更ログを記録する（tenant_id を含む）。
    async fn record_change_log(&self, log: &ConfigChangeLog) -> Result<(), ConfigRepositoryError> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(log.tenant_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;

        let start = std::time::Instant::now();
        sqlx::query(
            r#"
            INSERT INTO config_change_logs (
                id, tenant_id, config_entry_id, namespace, key, old_value_json, new_value_json,
                old_version, new_version, change_type, changed_by, trace_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(log.id)
        // ADR-0093 対応: config_change_logs.tenant_id は TEXT 型（migration 012）のため to_string() で TEXT として渡す
        .bind(log.tenant_id.to_string())
        .bind(log.config_entry_id)
        .bind(&log.namespace)
        .bind(&log.key)
        .bind(&log.old_value)
        .bind(&log.new_value)
        .bind(log.old_version)
        .bind(log.new_version)
        .bind(&log.change_type)
        .bind(&log.changed_by)
        .bind(&log.trace_id)
        .bind(log.changed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ConfigRepositoryError::Infrastructure(e.into()))?;
        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "record_change_log",
                "config_change_logs",
                start.elapsed().as_secs_f64(),
            );
        }

        Ok(())
    }
}
